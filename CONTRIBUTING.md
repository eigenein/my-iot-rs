## How to help?

I consider the service and message passing interface more-or-less stable and don't expect breaking changes. I'd be happy to receive a help in:

- Code review
- Performance improvements
- Cross-compilation for different boards
- New service implementation or API documentation
- Updating the book

## Workflow

- Latest code is always in `master`
- I commit directly to `master` because I don't care – and because I often run `make check` locally
- Feature branches get deleted after merging
- No release branches, flat commit history
- Releases get tagged with [annotated tags](https://git-scm.com/book/en/v2/Git-Basics-Tagging)
- `CHANGELOG.md` is really maintained
- [Semantic Versioning](https://semver.org/) is used, well… before `1.0.0` – not really
- Refer to issues in commit messages. Sometimes I personally forget to do that though

## Principles

- Clean code, because it's a learning project
- Stability, it should «just work»
- Easy installation on Raspberry Pi
- Good performance on [Raspberry Pi Zero W](https://www.raspberrypi.org/products/raspberry-pi-zero-w/)
- No plugins, [«batteries included»](https://en.wikipedia.org/wiki/Batteries_Included)
- Little configuration, good defaults, automatically discover peripherals and network devices where possible
- Unit tests where applicable, don't be obsessed with them though
- [Don't repeat yourself](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself) more than a few times
