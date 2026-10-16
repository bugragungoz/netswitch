// The built interface in Chromium against the simulated backend (src/api/mock.ts): every page
// opens without an error, the switch asks before it blocks, the Blocked page gives the internet
// back, filters and settings work, and the window has no sideways scroll at its smallest size.
//   npm run test:ui   (builds first; CHROMIUM_PATH picks a browser when Playwright finds none)
import assert from 'node:assert/strict';
import { after, before, test } from 'node:test';
import { launch } from './browser.mjs';
import { serve } from './server.mjs';

let server;
let browser;

before(async () => {
  server = await serve();
  browser = await launch();
});
after(async () => {
  await browser?.close();
  server?.close();
});

/** A fresh page in its own context (so settings saved in one test do not leak into the next). */
async function open({ width = 1280, height = 800, locale = 'en-US', colorScheme = 'dark' } = {}) {
  const context = await browser.newContext({ viewport: { width, height }, locale, colorScheme });
  const page = await context.newPage();
  const problems = [];
  page.on('pageerror', (e) => problems.push(e.message));
  page.on('console', (m) => {
    if (m.type() === 'error' || m.type() === 'warning') problems.push(m.text());
  });
  await page.goto(`${server.url}?quiet`);
  await page.getByRole('heading', { name: 'Overview' }).or(page.getByRole('heading', { name: 'Genel bakış' })).waitFor();
  return { page, context, problems };
}

const nav = (page, name) => page.getByRole('navigation').getByRole('button', { name: new RegExp(`^${name}`) }).click();

test('every page opens without an error', async () => {
  const { page, context, problems } = await open();
  await assert.doesNotReject(page.getByText('Demo data').waitFor());
  for (const [name, heading] of [
    ['Apps', 'Apps'],
    ['Connections', 'Connections'],
    ['Blocked', 'Blocked apps'],
    ['Settings', 'Settings'],
    ['Overview', 'Overview'],
  ]) {
    await nav(page, name);
    await page.getByRole('heading', { name: heading, exact: true }).waitFor();
  }
  assert.deepEqual(problems, []);
  await context.close();
});

test('the switch asks before it blocks, and the Blocked page gives the internet back', async () => {
  const { page, context, problems } = await open();
  await nav(page, 'Apps');
  const chrome = page.getByRole('switch', { name: 'Internet access for Google Chrome' });
  assert.equal(await chrome.getAttribute('aria-checked'), 'true');

  await chrome.click();
  const dialog = page.getByRole('dialog', { name: 'Block Google Chrome?' });
  await dialog.waitFor();
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await dialog.waitFor({ state: 'detached' });
  assert.equal(await chrome.getAttribute('aria-checked'), 'true');

  await chrome.click();
  await page.getByRole('dialog').getByRole('button', { name: 'Block', exact: true }).click();
  await page.getByText('Google Chrome is blocked.').waitFor();
  await page.waitForFunction(() => document.querySelector('[aria-label="Internet access for Google Chrome"]')?.getAttribute('aria-checked') === 'false');

  await nav(page, 'Blocked');
  const unblock = page.getByRole('button', { name: 'Unblock Google Chrome' });
  await unblock.click();
  await page.getByRole('dialog').getByRole('button', { name: 'Unblock', exact: true }).click();
  await page.getByText('Removed 2 rules.').waitFor();
  await unblock.waitFor({ state: 'detached' });
  assert.deepEqual(problems, []);
  await context.close();
});

test('a part of Windows cannot be switched off', async () => {
  const { page, context } = await open();
  await nav(page, 'Apps');
  const host = page.getByRole('switch', { name: 'Internet access for Host Process for Windows Services' });
  assert.equal(await host.isDisabled(), true);
  await context.close();
});

test('the app panel opens from a row and lists who the app talks to', async () => {
  const { page, context } = await open();
  await nav(page, 'Apps');
  await page.getByRole('searchbox').fill('steam');
  assert.equal(await page.getByRole('row').count(), 2); // the header and Steam
  await page.getByRole('row', { name: /Steam/ }).click();
  const panel = page.getByRole('complementary', { name: 'Steam' });
  await panel.waitFor();
  await panel.getByText('cache1-fra1.steamcontent.com').waitFor();
  await panel.getByRole('button', { name: 'Show connections' }).click();
  await page.getByRole('heading', { name: 'Connections', exact: true }).waitFor();
  assert.equal(await page.getByRole('searchbox').inputValue(), 'Steam');
  await context.close();
});

test('connection filters hide UDP and the local network', async () => {
  const { page, context } = await open();
  await nav(page, 'Connections');
  const table = page.getByRole('table', { name: 'Connections' });
  await table.waitFor();
  const cells = () => table.getByRole('cell').allInnerTexts();
  assert.ok((await cells()).some((c) => c.startsWith('UDP')));
  assert.ok((await cells()).includes('Local network') || (await table.getByText('Local network').count()) > 0);

  await page.getByRole('button', { name: 'UDP' }).click();
  await page.getByRole('switch', { name: 'Hide local network' }).click();
  await page.waitForTimeout(200);
  assert.ok(!(await cells()).some((c) => c.startsWith('UDP')));
  assert.equal(await table.getByText('Local network').count(), 0);
  await context.close();
});

test('language, theme and units change at once and are kept', async () => {
  const { page, context } = await open({ colorScheme: 'dark' });
  await nav(page, 'Settings');
  await page.getByRole('radio', { name: 'Light' }).click();
  assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), 'light');
  await page.getByRole('radio', { name: 'Bits (Mbit/s)' }).click();
  await page.getByRole('radio', { name: 'Türkçe' }).click();
  await page.getByRole('heading', { name: 'Ayarlar', exact: true }).waitFor();
  assert.equal(await page.evaluate(() => document.documentElement.lang), 'tr');

  await page.reload();
  await page.getByRole('heading', { name: 'Genel bakış' }).waitFor();
  assert.equal(await page.evaluate(() => document.documentElement.dataset.theme), 'light');
  assert.match(await page.locator('.stats').innerText(), /bit\/s/);
  await context.close();
});

test('the chart answers the keyboard', async () => {
  const { page, context } = await open();
  const plot = page.getByRole('img', { name: /Download and upload speed/ });
  await plot.focus();
  await page.keyboard.press('ArrowLeft');
  const tip = plot.getByRole('status');
  await tip.waitFor();
  assert.match(await tip.innerText(), /1 s ago/);
  await page.keyboard.press('Escape');
  await tip.waitFor({ state: 'detached' });
  await context.close();
});

test('the smallest window has no sideways scroll, on every page', async () => {
  const { page, context, problems } = await open({ width: 960, height: 600, locale: 'tr-TR' });
  for (const name of ['Genel bakış', 'Uygulamalar', 'Bağlantılar', 'Engellenenler', 'Ayarlar']) {
    await nav(page, name);
    await page.waitForTimeout(150);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    assert.equal(overflow, 0, `${name} scrolls sideways by ${overflow}px`);
  }
  assert.deepEqual(problems, []);
  await context.close();
});
