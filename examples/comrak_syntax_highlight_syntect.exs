Mix.install([
  {:mdex_native, path: Path.expand("..", __DIR__)}
])

markdown = """
# Syntect

```rust
impl<T> Option<T> {
    #[must_use]
    #[inline]
    pub const fn is_some_and(self, f: impl FnOnce(T) -> bool) -> bool {
        match self {
            None => false,
            Some(x) => f(x),
        }
    }

    #[inline]
    pub const fn as_ref(&self) -> Option<&T> {
        match *self {
            Some(ref x) => Some(x),
            None => None,
        }
    }

    #[inline]
    pub const fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }
}
```
"""

options = [
  syntax_highlight: [engine: :syntect, opts: [theme: "Catppuccin Macchiato"]]
]

markdown
|> MDExNative.Comrak.markdown_to_html(options)
|> IO.puts()
