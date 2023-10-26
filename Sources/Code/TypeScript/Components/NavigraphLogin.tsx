import { FSComponent, DisplayComponent, VNode, Subject, ComputedSubject, MappedSubject, Subscription, ComponentProps, EventBus } from '@microsoft/msfs-sdk';
import { CancelToken, DeviceFlowParams, User } from "navigraph/auth"
import { AuthService } from '../Services/AuthService';

interface NavigraphLoginProps extends ComponentProps {
    bus: EventBus;
}

export class NavigraphLogin extends DisplayComponent<NavigraphLoginProps> {
    
    private authParams = Subject.create<DeviceFlowParams | null>(null)
    private cancelSource = CancelToken.source()

    private verificationUrl = Subject.create<string>("");
  
    private authParamsSub?: Subscription

    private commBusListener: ViewListener.ViewListener;

    constructor(props: NavigraphLoginProps) {
        super(props);

        this.commBusListener = RegisterViewListener('JS_LISTENER_COMM_BUS', () => {
            console.log("JS_LISTENER_COMM_BUS registered");
        });

        this.commBusListener.on("NavdataUpdaterReceived", () => {
            console.log("WASM received request");
        })
    }

    public render(): VNode {
        return (
            <div>
                <div>{this.verificationUrl}</div>
                <img
                    src={MappedSubject.create(([verificationUrl]) => `https://api.qrserver.com/v1/create-qr-code/?size=500x500&data=${verificationUrl}`, this.verificationUrl)}
                />
            </div>
        );
    }

    public onAfterRender(node: VNode): void {
        super.onAfterRender(node);

        if (AuthService.getUser()) {
            console.log("already logged in!");
            console.log(AuthService.getUser());
        }

        this.authParamsSub = this.authParams.sub(p => {
            if (p) {
                this.verificationUrl.set(p.verification_uri_complete);
                console.log(p.verification_uri_complete)
            }
        });

        this.startDeviceFlow().then(() => {
            console.log("authenticated, trying to communicate with WASM");
            var jsonString = "{}";
            this.commBusListener.call("COMM_BUS_WASM_CALLBACK", "DownloadNavdata", jsonString);
        });
    }

    private startDeviceFlow() {
        this.cancelSource = CancelToken.source() // Reset any previous cancellations
        return AuthService.signIn(p => this.authParams.set(p), this.cancelSource.token)
      }
}