/// <reference types="@microsoft/msfs-types/Pages/VCockpit/Core/VCockpit" />

import { EventBus, FSComponent } from '@microsoft/msfs-sdk';
import { NavigraphLogin } from './Components/NavigraphLogin';
import { AuthService } from './Services/AuthService';

class MyInstrument extends BaseInstrument {
    private readonly bus: EventBus;

    constructor() {
        super();

        this.bus = new EventBus;
    }

    get templateID(): string {
        return 'MyInstrument';
    }

    public connectedCallback(): void {
        super.connectedCallback();

        AuthService.init(this.bus);

        FSComponent.render(<NavigraphLogin bus={this.bus} />, document.getElementById('InstrumentContent'));
    }

}

registerInstrument('my-instrument', MyInstrument);