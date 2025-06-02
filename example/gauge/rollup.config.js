import resolve from "@rollup/plugin-node-resolve";
import replace from "@rollup/plugin-replace";
import autoprefixer from "autoprefixer";
import dotenvSafe from "dotenv-safe";
import copy from "rollup-plugin-copy";
import esbuild from "rollup-plugin-esbuild";
import postcss from "rollup-plugin-postcss";
import tailwind from "tailwindcss";

dotenvSafe.config();

// eslint-disable-next-line no-undef
const DEBUG = process.env.DEBUG === "true";

let outputDest = "../aircraft/PackageSources";
if (DEBUG) {
  outputDest = "../aircraft/Packages/navigraph-aircraft-navigation-data-interface-sample";
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
      plugins: [tailwind, autoprefixer], // @ts-ignore
      use: { sass: { silenceDeprecations: ["legacy-js-api"] } },
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
};
