use std::collections::BinaryHeap;

use statrs::statistics::Statistics;

use crate::prelude::*;
use crate::services::anomaly::min_heap_reading::MinHeapReading;

/// [Normal distribution](https://en.wikipedia.org/wiki/Normal_distribution)-based
/// [anomaly detector](https://en.wikipedia.org/wiki/Anomaly_detection).
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct SimpleAnomalyDetector {
    sensor_id: String,

    sample_size: usize,

    /// <https://en.wikipedia.org/wiki/68%E2%80%9395%E2%80%9399.7_rule>
    #[serde(default = "default_sigma")]
    sigma: f64,
}

const fn default_sigma() -> f64 {
    3.0
}

impl SimpleAnomalyDetector {
    pub fn spawn(self, service_id: String, bus: &mut Bus, db: &Connection) -> Result {
        let rx = bus.add_rx();
        let tx = bus.add_tx();

        // Avoid selecting the entire sample from the database all the time.
        // The `self.sample_size`-th element will be pushed on the first iteration, thus the `-1`.
        let mut heap: BinaryHeap<MinHeapReading> = db
            .select_last_n_readings(&self.sensor_id, self.sample_size - 1)?
            .into_iter()
            .filter_map(|reading| {
                TryInto::<f64>::try_into(&reading.value)
                    .ok()
                    .map(|value| MinHeapReading(reading.timestamp, value))
            })
            .collect();

        thread::Builder::new().name(service_id.clone()).spawn(move || {
            let mut mean_variance: Option<(f64, f64)> = None;

            for message in rx {
                // TODO: duplicated code, see `threshold.rs`.
                if message.sensor.id != self.sensor_id {
                    continue;
                }
                let value = if let Ok(value) = TryInto::<f64>::try_into(&message.reading.value) {
                    value
                } else {
                    error!("[{}] Value is not `f64`.", &service_id);
                    continue;
                };

                if let Some((mean, variance)) = mean_variance {
                    let offset = (value - mean) / variance.sqrt();
                    debug!("[{}] {} | mean: {} | {:.2}σ", service_id, value, mean, offset);
                    let is_anomaly = offset.abs() > self.sigma;

                    let mut new_message = Message::new(format!("{}::{}::is_typical", service_id, self.sensor_id))
                        .timestamp(message.reading.timestamp)
                        .optional_location(message.sensor.location.clone())
                        .value(!is_anomaly);
                    if let Some(title) = &message.sensor.title {
                        new_message = new_message.sensor_title(format!("Is {} Typical", title));
                    }
                    new_message.send_and_forget(&tx);

                    if is_anomaly {
                        Message::new(format!("{}::{}::anomaly", service_id, self.sensor_id))
                            .type_(MessageType::ReadNonLogged)
                            .value(message.reading.value)
                            .timestamp(message.reading.timestamp)
                            .optional_sensor_title(message.sensor.title)
                            .optional_location(message.sensor.location)
                            .send_and_forget(&tx);
                    }
                } else {
                    debug!("[{}] The mean and variance have not been initialized yet.", service_id);
                }

                heap.push(MinHeapReading(message.reading.timestamp, value));
                if let Some((old_mean, old_variance)) = mean_variance {
                    // See also: https://jonisalonen.com/2014/efficient-and-accurate-rolling-standard-deviation/
                    let old_value = heap.pop().expect("the heap must be non-empty").1;
                    debug!("[{}] Popped value: {}", service_id, old_value);
                    let new_mean = old_mean + (value - old_value) / self.sample_size as f64;
                    let new_variance = old_variance
                        + (value - old_value) * (value - new_mean + old_value - old_mean)
                            / (self.sample_size - 1) as f64;
                    mean_variance = Some((new_mean, new_variance));
                } else {
                    debug!("[{}] Initializing the mean and variance…", service_id);
                    mean_variance = Some((
                        heap.iter().map(|reading| reading.1).mean(),
                        heap.iter().map(|reading| reading.1).variance(),
                    ));
                }
            }
        })?;

        Ok(())
    }
}
