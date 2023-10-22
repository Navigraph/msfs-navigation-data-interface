import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import copy from 'rollup-plugin-copy';
import css from 'rollup-plugin-import-css';

export default {
    input: 'MyInstrument.tsx',
    output: {
        dir: '../../../PackageSources/html_ui/Pages/VCockpit/Instruments/Navigraph/DataUpdaterSample',
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
                    dest: '../../../PackageSources/html_ui/Pages/VCockpit/Instruments/Navigraph/DataUpdaterSample',
                }
            ]
        })
    ]
}