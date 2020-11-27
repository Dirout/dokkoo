Chores:
* Keep documentation up to date & complete
* Maintain secure codebase
* Conform to Rust styleguide
* Complete all TODOs in codebase

WIP:
- [ ] Re-implement collections
    i.      When building, create collections: HashMap<collection: String, Vec<page: Page>>
    ii.     As compiling, append to collection's Vec<page>
    iii.    Recombine collection Vec<Page> into collections
    iv.     Write collections to binary file
    - [ ] Test current implementation
- [x] ~~Fully implement collections~~
    i.      As compiling, append to HashMap<collection: String, Vec<page: Page>>
    ii.     Incorporate into contexts as 'collections' context
    iii.    Compile files at root last
    - [x] Test current implementation
- [x] Look into replacing current Markdown processor
    * Very sensitive with whitespace
    * Performance could be improved
    - [x] Test replacement implementation
- [ ] Look into replacing Liquid crate(s)
    * Basic logic blocks, like 'if' statements and 'for' loops are not properly implemented
    * Performance could be improved
    - [ ] Test replacement implementation

Distribution plans:
* Consider iterating on branding prior to releases
* Linux-based operating systems (primary CLI release target)
    - [x] Binary
    - [x] Debian package
    - [x] RPM package
    - [ ] COPR
    - [] Gentoo
    - [x] Snapcraft
    - [ ] Flatpak
    - [x] Arch User Repository packages ('dokkoo-bin')
* Rust Crate (primary library release target)
    - [x] crates.io
* Unix-like operating systems
    - [ ] pkgsrc (planned; technically intended for NetBSD)
    - [ ] Nix
* Opensource BSD-based operating systems
    - [ ] FreeBSD Ports
    - [ ] OpenBSD Ports
* macOS & PureDarwin
    - [x] Binary
    - [ ] Homebrew (planned; technically available for Linux as well)
    - [ ] Fink (planned; technically available for Linux as well)
    - [ ] MacPorts
* Windows
    - [x] Binary
    - [ ] Chocolatey
    - [ ] Scoop