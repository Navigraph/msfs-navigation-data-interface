import resolve from "@rollup/plugin-node-resolve"
import replace from "@rollup/plugin-replace"
import dotenv from "dotenv"
import copy from "rollup-plugin-copy"
import esbuild from "rollup-plugin-esbuild"
import css from "rollup-plugin-import-css"

dotenv.config()

const DEBUG = process.env.DEBUG === "true"

let outputDest = "../aircraft/PackageSources"
if (DEBUG) {
  outputDest = "../aircraft/Packages/navigraph-aircraft-navdata-interface-sample"
}

export default {
  input: "MyInstrument.tsx",
  output: {
    dir: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/NavdataInterfaceSample`,
    format: "es",
  },
  plugins: [
    css({ output: "MyInstrument.css" }),
    resolve({ extensions: [".js", ".jsx", ".ts", ".tsx"] }),
    esbuild({ target: "es2017" }),
    replace({
      "process.env.NG_CLIENT_ID": JSON.stringify(process.env.NG_CLIENT_ID),
      "process.env.NG_CLIENT_SECRET": JSON.stringify(process.env.NG_CLIENT_SECRET),
      preventAssignment: true,
    }),
    copy({
      targets: [
        {
          src: "MyInstrument.html",
          dest: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/NavdataInterfaceSample`,
        },
      ],
    }),
  ],
}
