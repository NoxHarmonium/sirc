# VS Code support for the SIRC ecosystem

If you don't know what this is, check out the SIRC project at: https://github.com/NoxHarmonium/sirc

## Features

- Defines the SIRC flavour of assembly with the language ID 'sirc' and extension '.sasm'
- Adds syntax highlighting for SIRC flavoured assembly
- Adds the ability to use VS Code to debug SIRC programs running in sirc-vm (when the `--debug` switch is used).

## Usage

### Syntax Highlighting

It should associate any files with the `.sasm` extension with the SIRC assembly language (unless you have conflicting extensions).

### Debugging

Start sirc-vm in debug mode. For example:

```
cargo run --no-default-features --bin sirc_vm -- --debug -vv --program-file ./program.bin
```

The simulator will open a debug port and not start executing until a debugger is connected.

Then you'll want a `launch.json` file in the VS Code project with the assembly files you're running like the following:

```
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "sirc",
      "request": "attach",
      "name": "Attach to VM",
      "port": 9090
    }
  ]
}
```

Then you should be able to just press "F5" on the assembly file to start debugging.

## Local Development

Open the project folder in VS Code and press F5 to run it.

After pressing F5 another instance of VS Code will open with the plugin activated
and you can open the SIRC example projects to see the syntax highlighting
and debugging.
