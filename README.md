# chromatica
A Rust library for browser automation, based on CDP (Chrome DevTools Protocol).

# Coming soon
This library is already fully functional and actively used by me, though it requires further optimizations and some careful refactoring with `unsafe` for better performance.

The `examples` folder contains usage samples. Full documentation will be created later, once the core functionality is finalized and refined.

Currently, there is no implementation for launching the browser with custom parameters. I’m still considering how best to handle this, as the Puppeteer/Playwright/Chromiumoxide approach of searching and installing browsers upfront doesn’t seem suitable. Most likely, users will provide the path to the browser executable along with desired launch flags.
