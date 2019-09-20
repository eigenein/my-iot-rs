## Workflow

- Latest code is always in `master`
- I commit directly to `master` because I don't care (and because I run `make check` locally)
- Feature branches are deleted after merging
- Releases are tagged with [annotated tags](https://git-scm.com/book/en/v2/Git-Basics-Tagging)
- `CHANGELOG.md` is really maintained
- [Semantic Versioning](https://semver.org/) is used, almost

## Principles

- Clean code, because it's a learning project
- Stability, it should «just work»
- Easy installation on Raspberry Pi
- Good performance on [Raspberry Pi Zero W](https://www.raspberrypi.org/products/raspberry-pi-zero-w/)
- No plugins, [«batteries included»](https://en.wikipedia.org/wiki/Batteries_Included)
- Little configuration, good defaults, automatically discover peripherals and network devices
- Unit tests where applicable
- [Don't repeat yourself](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself)
