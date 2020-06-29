'use strict';

const cacheName = 'default';

// https://developers.google.com/web/ilt/pwa/caching-files-with-service-worker#network_falling_back_to_the_cache
self.addEventListener('fetch', function(event) {
  event.respondWith(
    caches.open(cacheName).then(function(cache) {
      return fetch(event.request).then(function (response) {
        console.log('📦 ' + event.request.url);
        cache.put(event.request, response.clone());
        return response;
      }).catch(function() {
        console.log('🎯 ' + event.request.url);
        return cache.match(event.request);
      });
    })
  );
});
