# Contributing

Any contribution is welcomed. <3

Bugs, fixes, ideas... anything.

> Nit: contributions for the sake of farming them are not well seen, while it is
> very much appreciated if you have found a typo or a one-line change, it is
> better to bring those changes along with a commit that changes that file.
> But it is very much encouraged to report everything that you see that needs a
> change.

## Getting started

```bash
git clone https://github.com/pabloosabaterr/vit
cd vit
cargo build --release
```

Wow, you've got your own clone of vit.

*Yes, there are not many tests...* But there are some benchmarks to know if it's
getting better or worse :).

you can run all of them with:

```bash
bash ./bench/all.sh
```

It should be a few seconds. After the first run an speedup % will appear
comparing the time with the last benchmark run.

## Discuss first

It is highly encouraged to discuss what you want to do before doing it, that
way we can think of the best solutions together. _team_work_.

While maybe a fix doesn't need many discussion and can opt to be sent directly
and then discussed, for features or things related with design is mandatory.

## AI guidelines

AI is not forbiden but take in mind that commits with no explanation whatsoever,
bloated or that the author is not able to explain what he has done ARE NOT
WELCOMED.

AI is encouraged to make the work of reviewing more easy (less typos, edge cases
coding guidelines).

You will often find better results if you ask AI to explain things and guide you
step by step until you get your solution.

## Ideas to contribute

### Performance optimizations

Mapping repositories it is a very exhaustive task.

```
- Git - 84891 commits

  map                    2896ms

- Rust - 335294 commits

  map                    9699ms
```

> RFC: more information about the mapping? docs about the math behind?

### New commands or options

Well I think this is very self explanatory :).

### Is there a way to update the mapping reliably without having to remap again?

Imagine:

```
                mapped at this point
                        v
<-----------------------*------>
      ^                    ^
 some commits          new commits
```

The word mapping depends on the frequency of the words across the whole repository
so adding new words from new commits will make that the frequencies of the words
that hasn't been updated to not represent reality reliably.

But how much damage does a word that appears once more when we have 80k commits and 10k
different words...

There should be margin for error where new commits can be processed without making
much noise.

It would be a huge milestone because the number of commits that needs to be
processed changes drastically.

## License

By contributing you agree your work is licensed under the same terms as the project (MIT OR Apache-2.0).
