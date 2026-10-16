// The README screenshots, from the simulated backend (made-up apps and documentation addresses).
//   npm run shots   ->  docs/images/*.png
import { mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { launch } from './browser.mjs';
import { serve } from './server.mjs';

const OUT = fileURLToPath(new URL('../../docs/images/', import.meta.url));
await mkdir(OUT, { recursive: true });
const server = await serve();
const browser = await launch();

async function shoot(name, { colorScheme = 'dark', steps = async () => {} } = {}) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, colorScheme, locale: 'en-US' });
  const page = await context.newPage();
  await page.goto(`${server.url}?quiet`);
  await page.getByRole('heading', { name: 'Overview' }).waitFor();
  // Let the charts fill with a few seconds of history.
  await page.waitForTimeout(2500);
  await steps(page);
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${OUT}${name}.png` });
  await context.close();
  console.log(`docs/images/${name}.png`);
}

const nav = (page, name) => page.getByRole('navigation').getByRole('button', { name: new RegExp(`^${name}`) }).click();

await shoot('overview');
await shoot('apps', {
  steps: async (page) => {
    await nav(page, 'Apps');
    await page.getByRole('row', { name: /Steam/ }).click();
  },
});
await shoot('connections', { colorScheme: 'light', steps: (page) => nav(page, 'Connections') });
await shoot('blocked', { colorScheme: 'light', steps: (page) => nav(page, 'Blocked') });

await browser.close();
server.close();
