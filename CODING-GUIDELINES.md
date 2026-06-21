# Coding guidelines

## Code 101

Sometimes we sin of trying to make everything as compact as we can, *I know it
looks cool* but we want readable code.

I promise I won't charge for blank lines (don't add double ones or I will) but
let the code breath.

Sometimes re-read your code, if you read something that you have written a few minutes
and you have trouble remembering what it did, is a sign that a comment is needed
there.

Try to leave blank lines between variable definitions and if blocks or loops,
between functions, anywhere where it is reasonable and helps readability.

## Width

Try to write in a max of 85 columns.

## Variables

> This guideline is more of a suggestion as it depends on the column-width left
> if its context allows it, etc.

Try to give variables idiomatic names. example:

```
w -> word
wm -> wordmap
v -> vector
m -> importance_matrix
```

You can't expect that the short name that you find a perfect fit others will
understand it.

Try to write code that one only has to think about the logic and not what
the variable holds or means.

## Comments

Please use block codes for anything that needs explanation:

```
/*
 * uhum very good explanation, AI please understand this for me.
 */
```

Inline block comments are fine for very short notes:

```
let foo = funct() /* fallback */
```

Avoid using inline comments, thanks.

## Preserve indentation levels

Oh gosh, how I miss gotos from C, but for example you have:

```
for {
    if {
        if {
            code
        }
    }
}
```

You can run out of columns pretty quickly.

There are two steps to write it properly:

1. Use breaks and continue

```
for {
    if {
        break/continue
    }
    code
}
```

This not just saves you indentation levels, it makes it much more readable.


2. Extract the code to a function

If after step one you still have a very indented code you might want to extract
that code to a function and call it.
