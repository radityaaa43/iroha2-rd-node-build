# iroha2-rd-node-build

Builds and tests the Iroha 2 node image of the RD/DFABI exploration on **GitHub-hosted runners** (this repo is public
for the 4 vCPU / 16 GB runner; it holds no keys, configs or cluster data). The network, deployment and findings live in
the private KPID repo, track `platforms/iroha2`.

| Path | What |
|---|---|
| `ci/prepare-src.sh` | upstream `hyperledger-iroha/iroha` @ `fbe677b9` + build fix + `image/Cargo.lock` + `fork/patches/*` + our crates |
| `fork/patches/` | P1 authority rules, P2 fail-closed BI disclosure envelope, P1b `vk_unshield` lock, (P3 `ConfidentialDvp`) |
| `driver/` | `rd2`: confidential wallet, BI audit, negative tests |
| `tools/rd-genesis/` | genesis v2 instructions |
| `image/build.sh` | node image: `iroha2d iroha2 kagami rd2 rd-genesis` |

Workflows: `ci` (every push: lockfile check, `rd_driver`/`rd_genesis` tests, `iroha_core` `rd_` tests), `build-image`
(push to `main` or manual: `ghcr.io/radityaaa43/iroha2-rd-node:fbe677b9-<commit>`, digest in the job summary).

Synthetic test data only. Not production software.
