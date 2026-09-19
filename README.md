# savers

<div align="center">

| Security Pillar | Verification Badge |
| :--- | :---: |
| **Platform Standard** | [![secured by studio2201][b-studio]][u-home] |
| **Credential Defense** | [![snip][b-snip]][u-snip] |
| **Supply Chain Surface** | [![vigil][b-vigil]][u-vigil] |
| **Post-Quantum Cryptography** | [![aegis][b-aegis]][u-aegis] |
| **Build Provenance & SLSA** | [![proven][b-proven]][u-proven] |
| **Repository Governance** | [![boneyard][b-boneyard]][u-boneyard] |

[b-studio]: https://img.shields.io/badge/secured%20by-studio2201-2f6f5e?logo=shield
[u-home]: https://studio2201.com
[b-snip]: https://img.shields.io/badge/snip-0%20secrets-2f6f5e?logo=shield
[u-snip]: https://studio2201.com/snip
[b-vigil]: https://img.shields.io/badge/vigil-0%20dependencies-2f6f5e?logo=shield
[u-vigil]: https://studio2201.com/vigil
[b-aegis]: https://img.shields.io/badge/aegis-PQC%20compliant-2f6f5e?logo=shield
[u-aegis]: https://studio2201.com/aegis
[b-proven]: https://img.shields.io/badge/proven-ML--DSA--65%20verified-2f6f5e?logo=shield
[u-proven]: https://studio2201.com/proven
[b-boneyard]: https://img.shields.io/badge/boneyard-maintained-2f6f5e?logo=shield
[u-boneyard]: https://studio2201.com/boneyard

</div>

All eleven official IdleScreen screensaver plugins in one workspace, plus
the `idle-savers` bundle (`meta/`). Part of
[IdleScreen](https://idlescreen.github.io) — modular Wayland screensavers
for Linux.

| Saver | Package | Description |
|---|---|---|
| `aurora/` | `idle-saver-aurora` | Aurora borealis curtains over a starfield (`[saver] aurora.*` params) |
| `beams/` | `idle-saver-beams` | Spotlight cones sweeping a rising dust starfield |
| `bursts/` | `idle-saver-bursts` | Firework rockets and particle bursts |
| `chaos/` | `idle-saver-chaos` | Strange-attractor particle chaos |
| `cosmos/` | `idle-saver-cosmos` | Accretion, ignition, and collapse of a tiny universe |
| `glyphs/` | `idle-saver-glyphs` | Falling luminous glyphs |
| `gnats/` | `idle-saver-gnats` | Swarming midges around a light |
| `hearth/` | `idle-saver-hearth` | Fireplace embers (`[saver] hearth.*` params) |
| `radar/` | `idle-saver-radar` | Radar sweep with drifting contacts |
| `ripple/` | `idle-saver-ripple` | Rain ripples on dark water |
| `storm/` | `idle-saver-storm` | Forest rain, lightning, wildlife silhouettes |

## Install

Ships with the `idlescreen` product package. On its own:

```sh
idlescreen install savers
```

## Commands

```sh
idlescreen preview storm   # try one fullscreen
idlescreen tui             # pick the active saver
idlescreen update          # get new saver builds
```

## License

Apache-2.0 · © 2026 IdleScreen

---

<div align="center">

[![Necrometer](necrometer.svg)](https://necrometer.dev/?u=idlescreen)

</div>
