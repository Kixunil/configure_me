Why this crate doesn't use derive
=================================

I'd definitely love to use derives, there's even [an issue](https://github.com/Kixunil/configure_me/issues/3) still open.

I currently see these obstacles around man page generation and other tooling (there's experimental `debconf` script generator already and I'd love to implement shell completions too):

* Man page generation was previously a part of the build script, but now it's moved to a separate tool to not mix up compilation with documenting. I was against this design (let people use cargo only, no makefiles) for some time, but eventually I started to like it more (separation of concerns).
* If I wanted to provide man page & co for derives, I'd either:
  * have to access files from within proc macros, this
    * goes against the design above
    * is extremely uncommon in Rust ecosystem (didn't see it anywhere at all)
    * might get impossible if syscalls from proc macros get disabled (I've read about some plans to do it)
  * scan the whole source tree from external tools, parse it and find all occurrences of my derive. I like this approach a bit, but see other problems with it:
    * Implementing seems to be very complex. I think I'd have to rely on someone else doing bulk of work, as I don't have too much time to implement such big things myself. According to https://github.com/alexcrichton/proc-macro2/issues/213 this feature doesn't exist yet.
    * obviously, when Rust adds a new syntax, the parsing instantly breaks, this doesn't happen with toml. To be fair, `syn` crate seems to be pretty up to date with the new stuff, so maybe not too hard to keep it up to date
    * what about other macros? If someone generates the struct with another macro, then the parser can't see it. Expanding the code doesn't help because it'd also expand my derive, losing the structural information.
    * I'm not sure how it'd work with interface files, which by definition have to be language-independent file format. I'm not pessimistic here, though.
    * scanning the whole tree might be slow in case of big projects (not sure).
  * do hacks like requiring people to have the struct in a single file that doesn't contain anything else (besides `use ...;`, I guess), this would be fairly easy, but lead to people being confused why man page generation doesn't work or even forgetting about it completely if they didn't attempt to generate man page at all. Then someone else would find it impossible to generate man page.
  * another hack would be to embed man page to the code and then the user could make the program itself output the man page. But what about other tools? I don't want to bake debconf script into every program...

I'm quite sad about this situation as I'd love to see less boilerplate (build.rs, which fortunately is currently only 3 lines), and proper IDE support. But considering the issues above, using toml seems to be a more useful trade-off to me. I hope you can see the reasons now. If you have a solution, I definitely want to know about it.
