import resolve from "@rollup/plugin-node-resolve"
import replace from "@rollup/plugin-replace"
import dotenv from "dotenv"
import copy from "rollup-plugin-copy"
import esbuild from "rollup-plugin-esbuild"
// import css from "rollup-plugin-import-css"
import postcss from "rollup-plugin-postcss"

dotenv.config()

// eslint-disable-next-line no-undef
const DEBUG = process.env.DEBUG === "true"

let outputDest = "../aircraft/PackageSources"
if (DEBUG) {
  outputDest = "../aircraft/Packages/navigraph-aircraft-navigation-data-interface-sample"
}

export default {
  input: "MyInstrument.tsx",
  output: {
    dir: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/NavigationDataInterfaceSample`,
    format: "es",
  },
  plugins: [
    resolve({ extensions: [".js", ".jsx", ".ts", ".tsx"] }),
    esbuild({ target: "es6" }),
    replace({
      // eslint-disable-next-line no-undef
      "process.env.NG_CLIENT_ID": JSON.stringify(process.env.NG_CLIENT_ID),
      // eslint-disable-next-line no-undef
      "process.env.NG_CLIENT_SECRET": JSON.stringify(process.env.NG_CLIENT_SECRET),
      preventAssignment: true,
    }),
    postcss({
      extract: true,
      minimize: true,
      output: "MyInstrument.css",
    }),
    copy({
      targets: [
        {
          src: "MyInstrument.html",
          dest: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/NavigationDataInterfaceSample`,
        },
      ],
    }),
  ],
}
