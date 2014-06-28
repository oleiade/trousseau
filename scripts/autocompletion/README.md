# Enhancing your trousseau experience with autocompletion

Thanks to the community, trousseau is shipped with a bunch of autocompletion scripts which will make your experience even more awesome.
Even if you don't know trousseau yet, your shell does and will be able to help you out.

## Zsh

To set up zsh autocompletion support for trousseau you will need to:

1. Setup an autocompletion scripts storage path. Generally `~/.zsh/completion` should make it. Anyway, feel free to use whatever you want and adapting the following procedure.
2. Copy the trousseau zsh autocompletion functions to this path:
    ```bash
    cp /your/cloned/trousseau/path/scripts/autocompletion/trousseau_autocomplete.zsh ~/.zsh/completion
    ```
3. Let zsh know about the completion path. This is done by setting the `$fpath` variable:
    ```bash
    fpath=(~/.zsh/completion $fpath)
    ```
4. Activate zsh completion system:
    ```bash
    autoload -U compinit
    compinit
    ```

Now everything should be working fine, even though you should automate these steps in your `~/.zshrc`

## Fish shell

### Installation and configuration

Just put it in your ``~/.config/fish/completions`` directory.
Yes, as simple as that.

## Bash

### Installation and configuration

Just put it in your ``/etc/bash_completion.d`` folder as ``trousseau`` for example.
That's it.

