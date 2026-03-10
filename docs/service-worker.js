// const CACHE_NAME = 'yew-pwa-cache-v1.01.03';
// const urlsToCache = [
//     'index.html',
//     'manifest.json',
//     'assets/croatian.json',
//     'pkg/yew_project_bg.wasm',
//     'pkg/yew_project.js',
//     'icons/icon-192x192.png',
//     'icons/icon-512x512.png',
//     'icons/icon-1024x1024.png',
//     'static/style.css',
// ];

// self.addEventListener('install', function(event) {
//     event.waitUntil(
//         caches.open(CACHE_NAME)
//             .then(cache => {
//                 // console.log('Service Worker: Caching Files');
//                 return cache.addAll(urlsToCache).catch(error => {
//                     console.error('Service Worker: Caching failed for one or more files:', error);
//                 });
//             })
//     );
// });

// self.addEventListener('fetch', event => {
//     if (event.request.url.includes('browser-sync')) {
//         console.log('Service Worker: Ignoring browser-sync request:', event.request.url);
//         return;  // Skip handling browser-sync requests
//     }
//     event.respondWith(
//         caches.match(event.request)
//             .then(response => {
//                 // Serve cached file if available
//                 if (response) {
//                     console.log('Service Worker: Fetching from cache:', event.request.url);
//                     return response;
//                 }
//                 // If not in cache, fetch from network
//                 console.log('Service Worker: Fetching from network:', event.request.url);
//                 return fetch(event.request)
//                     .then(networkResponse => {
//                         // Optionally cache the new file
//                         return caches.open(CACHE_NAME).then(cache => {
//                             cache.put(event.request, networkResponse.clone());
//                             return networkResponse;
//                         });
//                     })
//                     .catch(error => {
//                         console.error('Service Worker: Fetch failed; returning offline page', error);
//                         // You can add an offline fallback here if you have a dedicated offline page
//                         return caches.match('index.html');  // Fallback to main page or offline page
//                     });
//             })
//     );
// });


// self.addEventListener('activate', function(event) {
//     // console.log('Service Worker: Activated');
//     const cacheWhitelist = [CACHE_NAME];
//     event.waitUntil(
//         caches.keys().then(function(cacheNames) {
//             return Promise.all(
//                 cacheNames.map(cacheName => {
//                     if (!cacheWhitelist.includes(cacheName)) {
//                         console.log('Service Worker: Removing Old Cache:', cacheName);
//                         return caches.delete(cacheName);
//                     }
//                 })
//             );
//         })
//     );
//     self.clients.claim();
// });
