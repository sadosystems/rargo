# drop-point

drop point is a rustc wrapper designed specifically to work with abrasive. it provides caching on the crate level. The code is heavily inspired by sccache. Unlike sccache, drop-point only caches third-party / non workspace crates. drop-point assumes it is being run in a container (here container means a well defined environment with a digest). As such it doesn't go looking for for environment information when computing its hash.

drop point has just one CAS backend right now, a simple disk cache.