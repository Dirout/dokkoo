Chores:
* Keep documentation up to date & complete
* Maintain secure codebase
* Conform to Rust styleguide
* Complete all TODOs in codebase

WIP:
- [x] Fix bug where snippets in layouts can't access page values
- [x] Fix bug where snippets are processed an absurd amount of times
- [x] Fix bug where only the last snippet call is rendered unto the compiled output
- [ ] Pagination
- [ ] Plugins
- [ ] Proper async in CLI
- [ ] Move this all from a file to a GitHub Discussion

Distribution plans:
* Consider iterating on branding prior to releases
* Linux-based operating systems (primary CLI release target)
    - [x] Binary
    - [x] Debian package
    - [x] RPM package
    - [ ] COPR
    - [ ] Gentoo
    - [x] Snapcraft
    - [ ] Flatpak
    - [x] Arch User Repository packages ('dokkoo-bin')
* Rust Crate (primary library release target)
    - [x] crates.io
* Unix-like operating systems
    - [ ] pkgsrc (technically intended for NetBSD)
    - [ ] Nix
* Opensource BSD-based operating systems
    - [ ] FreeBSD Ports
    - [ ] OpenBSD Ports
* macOS & PureDarwin
    - [x] Binary
    - [ ] Homebrew (technically available for Linux as well)
    - [ ] Fink (technically available for Linux as well)
    - [ ] MacPorts
* Windows
    - [x] Binary
    - [ ] Chocolatey
    - [ ] Scoop