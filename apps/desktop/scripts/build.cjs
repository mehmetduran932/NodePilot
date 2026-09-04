const crypto = require('crypto');
if (crypto.webcrypto) {
  globalThis.crypto = crypto.webcrypto;
  crypto.getRandomValues = (arr) => crypto.webcrypto.getRandomValues(arr);
}

import('vite').then(({ build }) => {
  build().then(() => {
    console.log('Frontend built successfully!');
  }).catch(err => {
    console.error(err);
    process.exit(1);
  });
});
