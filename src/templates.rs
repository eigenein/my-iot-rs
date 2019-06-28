//! Describes all the templates.

use crate::measurement::Measurement;

markup::define! {
    Base {
        {markup::doctype()}
        html[lang = "en"] {
            head {
                title { "My IoT" }
                meta[charset = "utf-8"];
                meta[name = "viewport", content = "width=device-width, initial-scale=1"];
                meta["http-equiv" = "refresh", content = "60"];
                link[rel = "stylesheet", href = "https://cdnjs.cloudflare.com/ajax/libs/bulma/0.7.4/css/bulma.min.css"];
                link[rel = "stylesheet", href = "https://use.fontawesome.com/releases/v5.5.0/css/all.css"];
                script[src = "https://cdn.plot.ly/plotly-1.5.0.min.js"] {}
            }
            body {
                footer.footer {
                    div.container {
                        div.columns {}
                    }
                }
                script {
                    {markup::raw(r#"
                        /*
                         * Used the implementation example from:
                         * https://bulma.io/documentation/components/navbar/
                         */

                        document.addEventListener('DOMContentLoaded', () => {
                            const $navbarBurgers = Array.prototype.slice.call(document.querySelectorAll('.navbar-burger'), 0);
                            if ($navbarBurgers.length > 0) {
                                $navbarBurgers.forEach(el => {
                                    el.addEventListener('click', () => {
                                        const $target = document.getElementById(el.dataset.target);
                                        el.classList.toggle('is-active');
                                        $target.classList.toggle('is-active');
                                    });
                                });
                            }
                        });
                    "#)}
                }
            }
        }
    }
}

markup::define! {
    Index {
        {Base {}}
    }
}
