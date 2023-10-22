import { FSComponent, DisplayComponent, VNode, Subject, ComputedSubject, MappedSubject } from '@microsoft/msfs-sdk';
import { User } from "navigraph/auth"
import { auth } from "../lib/navigraph";

export class NavigraphLogin extends DisplayComponent<any> {
    private user: User | null = null;

    private verificationUrl = Subject.create<string>("");
    private loginText = ComputedSubject.create(false, (isLoggingIn) => isLoggingIn ? "Logging in..." : "Log in");

    public render(): VNode {
        return (
            <div>
                <div>{this.loginText}</div>
                <img
                    src={MappedSubject.create(([verificationUrl]) => `https://api.qrserver.com/v1/create-qr-code/?size=500x500&data=${verificationUrl}`, this.verificationUrl)}
                />
            </div>
        );
    }

    public onAfterRender(node: VNode): void {
        super.onAfterRender(node);

        auth.signInWithDeviceFlow((deviceFlowParams) => {
            this.verificationUrl.set(deviceFlowParams.verification_uri_complete);
            this.loginText.set(true);
        });
    }
}