import type { CodeStageDemo } from "../components/code-stage";

/** Homepage tabbed demos — short, identity-carrying samples. */
export const HOME_DEMOS: CodeStageDemo[] = [
  {
    id: "leaders",
    label: "Leaders",
    blurb:
      "Statements start with a glyph. Bind, mutate, branch, and loop without English keywords.",
    code: `/ std/io

$ xs = [1, 2, 3]
~ sum = 0
* x : xs {
    ~ sum = sum + x
}
io.print("sum={sum}")
`,
    command: "xo run sum.echo",
    output: "sum=6",
    docsHref: "/docs/leaders",
    docsLabel: "Leaders reference",
  },
  {
    id: "result",
    label: "Result",
    blurb:
      "Errors are values. ! returns an err; | matches ok and err. The program does not abort on !.",
    code: `/ std/io
/ std/str

$ checked = (x) {
    ? x < 0 {
        ! 99
    }
    ^ x
}

| checked(7) {
    $ v {
        io.print(str.from_int(v))
    }
    ! e {
        io.print(str.from_int(e))
    }
}
`,
    command: "xo run result.echo",
    output: "7",
    docsHref: "/docs/result-option",
    docsLabel: "Result and option",
  },
  {
    id: "structs",
    label: "Structs",
    blurb: "% declares a shape. Fields use $ / ~ / #; methods take the receiver as .",
    code: `/ std/io
/ std/str

% point {
    ~ x
    ~ y
}

$ p = point { x: 3, y: 4 }
io.print(str.from_int(p.x))
~ p.x = p.x + 10
io.print(str.from_int(p.x))
`,
    command: "xo run point.echo",
    output: "3\n13",
    docsHref: "/docs/structs",
    docsLabel: "Structs reference",
  },
  {
    id: "tasks",
    label: "Tasks",
    blurb: "+ spawns work; - joins or runs a block. Handles are values you wait on.",
    code: `/ std/io
/ std/str

+ job = {
    ^ 7
}
- v = job
io.print(str.from_int(v))
`,
    command: "xo run task.echo",
    output: "7",
    docsHref: "/docs/tasks",
    docsLabel: "Tasks reference",
  },
];

/** Sample cards linking into the tree / cookbook after install. */
export const HOME_SAMPLES: {
  title: string;
  path: string;
  blurb: string;
  href: string;
}[] = [
  {
    title: "hello",
    path: "examples/misc/hello.echo",
    blurb: "Print via std — the smallest run.",
    href: "/docs/first-program",
  },
  {
    title: "sum list",
    path: "examples/misc/sum_list.echo",
    blurb: "Leaders, loop, and mutable bind.",
    href: "/docs/first-program",
  },
  {
    title: "result",
    path: "examples/misc/result_ok.echo",
    blurb: "Match an ok path with |.",
    href: "/docs/result-option",
  },
  {
    title: "point",
    path: "examples/misc/point.echo",
    blurb: "Named struct fields and assign.",
    href: "/docs/structs",
  },
  {
    title: "counter",
    path: "examples/misc/counter.echo",
    blurb: "Methods and receiver .",
    href: "/docs/structs",
  },
  {
    title: "cookbook",
    path: "docs/guides",
    blurb: "Recipes once you have xo.",
    href: "/docs/guides/cookbook",
  },
];
