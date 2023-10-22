/// <reference types="@microsoft/msfs-types/Pages/VCockpit/Core/VCockpit" />

import { FSComponent } from '@microsoft/msfs-sdk';
import { MyComponent } from './MyComponent';
import { NavigraphLogin } from './Components/NavigraphLogin';

class MyInstrument extends BaseInstrument {
    get templateID(): string {
        return 'MyInstrument';
    }

    get isInteractive(): boolean {
        return true;
    }

    public connectedCallback(): void {
        super.connectedCallback();

        FSComponent.render(<NavigraphLogin />, document.getElementById('InstrumentContent'));
    }
}

registerInstrument('my-instrument', MyInstrument);