/* eslint-disable @typescript-eslint/class-literal-property-style */
/// <reference types="@microsoft/msfs-types/pages/vcockpit/instruments/shared/baseinstrument" />
/// <reference types="@microsoft/msfs-types/pages/vcockpit/core/vcockpit" />

import { EventBus, FSComponent } from "@microsoft/msfs-sdk";
import { InterfaceSample } from "./Components/InterfaceSample";
import { AuthService } from "./Services/AuthService";
import "./MyInstrument.css";

class MyInstrument extends BaseInstrument {
  private readonly bus: EventBus;

  constructor() {
    super();

    this.bus = new EventBus();
  }

  get templateID(): string {
    return "MyInstrument";
  }

  get isInteractive(): boolean {
    return true;
  }

  public connectedCallback(): void {
    super.connectedCallback();

    AuthService.init(this.bus);
    FSComponent.render(<InterfaceSample bus={this.bus} />, document.getElementById("InstrumentContent"));
  }
}

registerInstrument("my-instrument", MyInstrument);
