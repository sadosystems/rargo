I have never had all that much luck with tests, except for the extensive suite of 
hypothesis based tests I work with / on at $job. I have had the most success with
tests that are very data driven and treat the implementation mostly as a black box.
the matklad essay on basically this is very very good
https://matklad.github.io/2021/05/31/how-to-test.html
I would like to take this project as an opportunity to excercise some of these
principles, as well as the this essay:
https://matklad.github.io/2024/01/03/of-rats-and-ratchets.html

In particular I think it would be good to write down some invariants that I think 
should hold for a completed version of this project, then write some tests against a
dream api / the application itself so I can see them all fail, THEN I want to write
an allow list for basically all the tests and start plugging away one by one on making 
all the tests pass. As shameful as it is to admit I care about this kind of thing I want 
to do this in such a way that the failing tests are actually red and failing (not green
and passing due to properly failing) I guess the mechanism for that is to make a list
of required tests.

the most top level invariant is the idea that all commands from cargo "work" meaning 
they do the same thing as the cargo equivalent. How to test that? the stdout isnt 
Exactly the same. hmmm

also the sans IO idea really struck me. I have no clue how to make this sans IO. 
what we have here is basically microservices...

I think what I want is some defined state machines and an event like system similar
to what I did for the bus orchestrator, everything is 
(State, Event, T(or delta_t)) -> State2, Effect
then I can write a sim that runs all my little micro services or whatever in one 
process and have control over scheduling... but that leaves a big open question for me
how do I model or mock rustc and cargo in these tests? do I even model / mock them?

I did a little bit of an ADHD research binge, here are the loose ends from that:
https://lib.rs/crates/arbitrary
https://buttondown.com/hillelwayne/archive/cross-branch-testing/
https://sans-io.readthedocs.io/
https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/
