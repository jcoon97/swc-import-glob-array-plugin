import md from "marked";
import { dir as _iga1 } from "./docs/hello.mdx";
import { dir as _iga2 } from "./docs/world.mdx";
const foo = [
    _iga1,
    _iga2
];
import _iga3, { foo as _iga4 } from "./docs/hello.mdx";
import _iga5, { foo as _iga6 } from "./docs/world.mdx";
const bar = [
    _iga4,
    _iga6
];
const wow = [
    _iga3,
    _iga5
];
const meta = [
    {
        absolutePath: "$DIR/tests/fixtures/basic/docs/hello.mdx",
        importedPath: "./docs/hello.mdx"
    },
    {
        absolutePath: "$DIR/tests/fixtures/basic/docs/world.mdx",
        importedPath: "./docs/world.mdx"
    }
];
