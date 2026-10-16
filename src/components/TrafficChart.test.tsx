import { fireEvent, render, screen } from '@testing-library/preact';
import { describe, expect, it } from 'vitest';
import { setLanguage } from '../i18n';
import { TrafficChart } from './TrafficChart';

const history = [
  { in: 1000, out: 100 },
  { in: 2000, out: 200 },
  { in: 4000, out: 300 },
];

describe('TrafficChart', () => {
  it('names both series and the latest values in the legend and for screen readers', () => {
    setLanguage('en');
    const { container } = render(<TrafficChart history={history} span={300} unit="bytes" label="Speed" />);
    const legend = container.querySelector('.chart-legend')!;
    expect(legend.textContent).toContain('Download');
    expect(legend.textContent).toContain('4.0 KB/s');
    expect(legend.textContent).toContain('Upload');
    expect(legend.textContent).toContain('300 B/s');
    expect(screen.getByRole('img').getAttribute('aria-label')).toBe('Speed. Download 4.0 KB/s, Upload 300 B/s. Peak 4.0 KB/s.');
  });

  it('moves a crosshair with the arrow keys and hides it on Escape', () => {
    setLanguage('en');
    render(<TrafficChart history={history} span={300} unit="bytes" label="Speed" />);
    const plot = screen.getByRole('img');
    fireEvent.keyDown(plot, { key: 'ArrowLeft' });
    const tip = screen.getByRole('status');
    expect(tip.textContent).toContain('1 s ago');
    expect(tip.textContent).toContain('2.0 KB/s');
    expect(tip.textContent).toContain('200 B/s');
    fireEvent.keyDown(plot, { key: 'Home' });
    expect(screen.getByRole('status').textContent).toContain('2 s ago');
    expect(screen.getByRole('status').textContent).toContain('1.0 KB/s');
    fireEvent.keyDown(plot, { key: 'Escape' });
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('draws a small chart without axes, legend or crosshair', () => {
    const { container } = render(<TrafficChart history={history} span={60} unit="bytes" label="Speed" compact />);
    expect(container.querySelector('.chart-legend')).toBeNull();
    expect(container.querySelector('.chart-tick')).toBeNull();
    fireEvent.keyDown(screen.getByRole('img'), { key: 'ArrowLeft' });
    expect(screen.queryByRole('status')).toBeNull();
  });
});
