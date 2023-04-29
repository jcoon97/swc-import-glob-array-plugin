# SWC Plugin Import Glob Array

This plugin is a port of [babel-plugin-import-glob-array](https://github.com/jescalan/babel-plugin-import-glob-array)
from JavaScript to Rust for [Speedy Web Compiler (SWC)](https://swc.rs/).

Before continuing, please understand that SWC plugins are distributed as WebAssembly binaries and
are [currently not backwards compatible](https://swc.rs/docs/plugin/selecting-swc-core), meaning that plugins may break
frequently between versions. To ensure that you're using a version of this plugin that is compatible with your
current `@swc/core` or `next` package(s), check out the [compatibility](#compatibility) table below.

## Installation

Installation can be done via npm:

```shell
# npm
$ npm install --save-dev swc-plugin-import-glob-array

# yarn
$ yarn add -D swc-plugin-import-glob-array

# pnpm
$ pnpm add -D swc-plugin-import-glob-array
```

Then you'll add the plugin under `jsc.experimental.plugins`:

```json
{
  "jsc": {
    "experimental": {
      "plugins": [
        [
          "swc-plugin-import-glob-array",
          {}
        ]
      ]
    }
  }
}
```

For more information on adding and configuration an SWC plugin, check 
out [their documentation](https://swc.rs/docs/configuration/compilation).

## Usage

This plugin check the imports of each SWC file to determine if it contains a glob pattern. If it does, it will expand 
the single import to individual imports. As an example, let's say you have the following directory structure:

```
|- my-app
  |- docs
    |- hello.md
    |- world.md
  |- index.js
```

And `index.js`, which will get compiled by SWC, has the following import:

```js
import docs from "./docs/*.md";
```

The single import will get parsed and expanded into:

```js
import _iga1 from "./docs/hello.md";
import _iga2 from "./docs/world.md";

const docs = [ _iga1, _iga2 ];
```

### Adding Import Metadata

In addition to expanding a single import, you can also import metadata information about where the file came from and 
what was imported via the special `_importMeta` property.

Using the example above, we can slightly modify our import statement:

```js
import docs, { _importMeta as metadata } from "./docs/*.md";
```

Which will get parsed and expanded into:

```js
import _iga1 from "./docs/hello.md";
import _iga2 from "./docs/world.md";

const docs = [ _iga1, _iga2 ];

const metadata = [
    {
        absolutePath: "/path/to/project/docs/hello.md",
        importedPath: "./docs/hello.md"
    },
    {
        absolutePath: "/path/to/project/docs/world.md",
        importedPath: "./docs/world.md"
    }
];

```

## Compatibility

| swc-plugin-import-glob-array | @swc/core     | next   |
|------------------------------|---------------|--------|
| 1.0.0                        | 1.3.44-1.3.47 | 13.3.1 |
