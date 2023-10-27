import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import copy from 'rollup-plugin-copy';
import css from 'rollup-plugin-import-css';

let DEBUG = true;

let outputDest = '../../../PackageSources';
if (DEBUG) {
    outputDest = '../../../Packages/navigraph-aircraft-updater-sample'
}

export default {
    input: 'MyInstrument.tsx',
    output: {
        dir: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/DataUpdaterSample`,
        format: 'es'
    },
    plugins: [
        css({ output: 'MyInstrument.css' }),
        resolve(),
        typescript(),
        copy({
            targets: [
                {
                    src: 'MyInstrument.html',
                    dest: `${outputDest}/html_ui/Pages/VCockpit/Instruments/Navigraph/DataUpdaterSample`,
                }
            ]
        })
    ]
}