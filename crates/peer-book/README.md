# Peer Book

A contact book app for your peers.

Look up peer by VLAD, see their available apps and data. 

Use Peer Book to grant access to others to yuor data and apps.

Uses pernames to associate VLADs to something or someone.

## Tutorial: How this plugin was created:

- `cargo component new --lib peer-book` creates a new wasm WIT component using [cargo-component](https://github.com/bytecodealliance/cargo-component).
- remove `[profile.release]` from Cargo.toml as we have a workspace 
- add directory `./wit/deps/` for a symlink to the host interface at `../../wit/host.wit`
- create symlink with `ln -s ../../wit/host.wit ./wit/deps/host.wit` to give us the host interface
- add `import host:component/host;` to our local `world` in `./wit/world.wit` to bring host interface into our world scope
- in order for the `./wit/deps/host.wit` to be found by the `wit` compiler, we need to add the following to the `Cargo.toml` file:

```Cargo.toml
[package.metadata.component.target.dependencies]
"host:component" = { path = "wit/deps" }
```

- our render process relies on all plugins having the same "package" name, so change the default component name to:

```world.wit
package component:plugin; // was package component:peer-book 
```

- add `export run;` to our local `world` in `./wit/world.wit` to export our run function interface 
- at a minimum, add a load function to the `run` interface in `./wit/run.wit` to load the RDX for rendering:

```wit
interface run {
  
  /// loads just the RDX for rendering
  load: func() -> string;

}

world example {
  
  import host:component/host;

  /// export our run interface
  export run;
}
```

- once the deps are set, we will need to update our Rust code to reflect the new interface.

```rust

use bindings::exports::component::plugin::run::Guest; // update path to exported run interface

/// Generate some basic RDX for rendering
fn load() -> String {
        r#"
        render(`
            <div>
                <label>Peer Book</label>

                <!-- We bind the value of vlad to a rhai::Scope variable we named `vlad` -->
                <input value="{{vlad}}" />

                <!-- because we defined a {{vlad}} above, it is available to any RDX function in the html -->
                <button data-on-click="search(vlad)">Search</button>
            </div>
        `)
        "#
        .to_string()    
}
```

Note that valid html is required, but only a subset of html is supported by the RDX renderer at this time. Invalid html will likely cause the RDX renderer to fail. Given that, it's probably best to use the [html](https://docs.rs/html/latest/html/) crate and a `build.rs` process to generate your RDX at compile time, so you can benefit from type safety to avoid typos, and precompile the Rhai to ensure it is also valid.

- addtional functions and types can be added to `run` as needed, and used by any RDX template code. The parsed RDX renderer will automatically search for named functions in the wasm code it finds int he RDX template. For example, in our new code `on_click` will try to find an exported func name `search` with the rhai scope value under `vlad` as the argument.
- `search(vlad)` doesn't exist yet, but we can add it to both our interface (WIT) and our Rust code:
    
```wit
interface run {
  
  /// loads just the RDX for rendering
  load: func() -> string;

  /// search for a peer by VLAD
  search: func(vlad: string) -> string;

}
```
- as we build our Rust implementation, it can be useful have see log outputs, which is provided to use by the host interace. We just need to import it:

```rust 
use crate::bindings::host::component::host::log;


impl Guest for Component {
    /// Say hello!
    fn load() -> String {
        // .. snip
    }

    /// Search for a peer by VLAD 
    fn search(vlad: String) -> String {
        log(&format!("searching for peer: {}", vlad));
        format!("Plog results for vlad: {}", vlad)
    }
```

- Now that the function is scaffolded, we can add the actual search logic to our Rust code. The actual results are put into the Blockstore by the bestsign plugin, and since we both have bitswap and are connected, all we need to do it send a `Get` command and PeerPiper will look in the local blockstore first, then try bitswap with peers we're connected to in order to find the Vlad head CID bytes. So let's send a command to order a `Get`.

```rust 
commander.order(AllCommands::Get { key: vlad.head.into() });
```

Since the results of order are async, which would block our egui, we simply load the results into the rhai scope for the plugin when they are ready. The plugin displays the rhai scope values once they are available. 

```rust
    /// Search for a peer by VLAD 
    fn search(vlad: String) -> String {
        log(&format!("searching for peer: {}", vlad));
        commander.order(AllCommands::Get { key: vlad.head.into() });
        "Searching...".to_string()
    }
```
