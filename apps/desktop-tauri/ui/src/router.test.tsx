import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { userEvent } from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { AppLayout } from './router.js';
import { MockDesktopClient } from './api/client.js';

describe('desktop ui routes', () => {
  it('renders sessions table success state', async () => {
    render(
      <MemoryRouter initialEntries={['/sessions']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    expect(await screen.findByText('Sessions')).toBeTruthy();
    expect(await screen.findByText('sess_mock_001')).toBeTruthy();
  });

  it('navigates from findings evidence to session route', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={['/findings']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    const openEvidenceButton = await screen.findByRole('button', { name: /Open Evidence 1/i });
    await user.click(openEvidenceButton);

    await waitFor(() => {
      expect(screen.getByText(/Session: sess_mock_001/i)).toBeTruthy();
    });
  });

  it('shows export actions with full export policy gating', async () => {
    render(
      <MemoryRouter initialEntries={['/exports']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    const shareSafe = await screen.findByRole('button', { name: /Generate Share-Safe Export/i });
    const full = await screen.findByRole('button', { name: /Generate Full Export/i });

    expect(shareSafe.getAttribute('disabled')).toBeNull();
    expect(full.getAttribute('disabled')).not.toBeNull();
  });

  it('shows live capture pairing diagnostics', async () => {
    render(
      <MemoryRouter initialEntries={['/live-capture']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    expect(await screen.findByText(/Status: connected/i)).toBeTruthy();
    expect(await screen.findByText(/Pairing port: 32124/i)).toBeTruthy();
    expect(
      await screen.findByText(/Pairing token: 0123456789abcdef0123456789abcdef/i),
    ).toBeTruthy();
  });

  it('applies highlight class for 4 seconds on exact evidence target', async () => {
    const { container } = render(
      <MemoryRouter
        initialEntries={[
          '/sessions/sess_mock_001/network?hl_kind=net_row&hl_id=net_mock_1&hl_col=status_code&hl_ptr=%2Fstatus_code&hl_exact=true',
        ]}
      >
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    await screen.findByText(/Session: sess_mock_001/i);
    const target = await waitFor(() =>
      container.querySelector('[data-highlight-key="net_row:net_mock_1:status_code:/status_code"]'),
    );
    expect(target).toBeTruthy();
    expect(target?.classList.contains('pulse-highlight')).toBe(true);

    await new Promise((resolve) => {
      window.setTimeout(resolve, 4100);
    });
    expect(target?.classList.contains('pulse-highlight')).toBe(false);
  });

  it('shows fallback notice when exact pointer is unavailable', async () => {
    render(
      <MemoryRouter
        initialEntries={[
          '/sessions/sess_mock_001/network?hl_kind=net_row&hl_id=net_mock_1&hl_col=status_code&hl_ptr=%2Fmissing&hl_exact=false&hl_fallback=Exact%20pointer%20unavailable',
        ]}
      >
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    expect(await screen.findByText(/Exact pointer unavailable/i)).toBeTruthy();
  });

  it('runs release dry-run from exports screen', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={['/exports']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    const releaseButton = await screen.findByRole('button', {
      name: /Start Internal Beta Release \(Dry Run\)/i,
    });
    await user.click(releaseButton);

    expect(await screen.findByText(/Release rel_mock_1 status: completed/i)).toBeTruthy();
    expect(await screen.findByText(/Platform Artifact Matrix/i)).toBeTruthy();
    expect(await screen.findByText(/windows_zip/i)).toBeTruthy();
  });

  it('shows staged promotion gate as disabled when smoke evidence is missing', async () => {
    render(
      <MemoryRouter initialEntries={['/exports']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    const promotionButton = await screen.findByRole('button', {
      name: /Start Staged Public Promotion \(Dry Run\)/i,
    });
    expect(promotionButton.getAttribute('disabled')).not.toBeNull();
    expect(await screen.findByText(/manual smoke/i)).toBeTruthy();
  });

  it('renders extension rollout and updater controls', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={['/exports']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole('button', {
        name: /Start Extension Public Rollout \(Dry Run\)/i,
      }),
    ).toBeTruthy();
    const checkUpdates = await screen.findByRole('button', { name: /Check for Updates/i });
    await user.click(checkUpdates);
    expect(await screen.findByText(/Update check: eligible/i)).toBeTruthy();
  });

  it('opens bundle inspect route from exports screen', async () => {
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={['/exports']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    const bundleInput = await screen.findByPlaceholderText(/\/absolute\/path\/to\/export.zip/i);
    await user.type(bundleInput, '/tmp/mock-export.zip');

    const openInspectButton = await screen.findByRole('button', {
      name: /Open Bundle Inspect/i,
    });
    await user.click(openInspectButton);

    expect(await screen.findByText(/Bundle Inspect/i)).toBeTruthy();
    expect(await screen.findByText(/Integrity: true/i)).toBeTruthy();
  });

  it('renders diagnostics reliability and perf controls', async () => {
    render(
      <MemoryRouter initialEntries={['/diagnostics']}>
        <AppLayout client={new MockDesktopClient()} />
      </MemoryRouter>,
    );

    expect(await screen.findByText(/Reliability KPIs \(24h\)/i)).toBeTruthy();
    expect(
      await screen.findByRole('button', { name: /Start Sustained Capture Perf/i }),
    ).toBeTruthy();
    expect(await screen.findByRole('button', { name: /Run Telemetry Export/i })).toBeTruthy();
    expect(await screen.findByRole('button', { name: /Run Telemetry Audit/i })).toBeTruthy();
  });

  it('keeps diagnostics page accessible when some backend calls fail', async () => {
    class PartialFailureClient extends MockDesktopClient {
      override async uiGetReliabilitySnapshot(_windowMs: number): Promise<never> {
        throw new Error('simulated diagnostics backend failure');
      }
    }

    render(
      <MemoryRouter initialEntries={['/diagnostics']}>
        <AppLayout client={new PartialFailureClient()} />
      </MemoryRouter>,
    );

    expect(await screen.findByText(/About \/ Diagnostics/i)).toBeTruthy();
    expect(await screen.findByText(/Some diagnostics data is unavailable/i)).toBeTruthy();
    expect(await screen.findByText(/Pairing port:/i)).toBeTruthy();
  });
});
