import { ComponentProps, DataStore, DisplayComponent, EventBus, FSComponent, VNode } from "@microsoft/msfs-sdk"
import { CancelToken } from "navigraph/auth"
import { AUTH_STORAGE_KEYS } from "../Lib/navigraph"
import { AuthService } from "../Services/AuthService"
import "./NavigraphLogin.css"

interface NavigraphLoginProps extends ComponentProps {
  bus: EventBus
}

export class NavigraphLogin extends DisplayComponent<NavigraphLoginProps> {
  private readonly textRef = FSComponent.createRef<HTMLDivElement>()
  private readonly buttonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly qrCodeRef = FSComponent.createRef<HTMLImageElement>()

  private cancelSource = CancelToken.source()

  private commBusListener: ViewListener.ViewListener

  constructor(props: NavigraphLoginProps) {
    super(props)

    this.commBusListener = RegisterViewListener("JS_LISTENER_COMM_BUS", () => {
      console.info("JS_LISTENER_COMM_BUS registered")
    })

    this.commBusListener.on("NavdataUpdaterReceived", () => {
      console.info("WASM received request")
    })
  }

  public render(): VNode {
    return (
      <div class="auth-container">
        <div ref={this.textRef} />
        <div ref={this.buttonRef} onClick={this.handleClick.bind(this)} class="login-button" />
        <img ref={this.qrCodeRef} class="qr-code" />
      </div>
    )
  }

  public onBeforeRender(): void {
    super.onBeforeRender()
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.buttonRef.instance.addEventListener("click", () => this.handleClick())

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = ""
        this.qrCodeRef.instance.style.display = "none"
        this.buttonRef.instance.textContent = "Log out"
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`
      } else {
        this.buttonRef.instance.textContent = "Sign in"
        this.textRef.instance.textContent = "Not signed in"
      }
    }, true)
  }

  private handleClick() {
    this.commBusListener.call("COMM_BUS_WASM_CALLBACK", "DownloadNavdata", "{}")
    if (AuthService.getUser()) {
      void AuthService.signOut()
    } else {
      this.cancelSource = CancelToken.source() // Reset any previous cancellations
      AuthService.signIn(p => {
        if (p) {
          this.qrCodeRef.instance.src = `https://api.qrserver.com/v1/create-qr-code/?size=500x500&data=${p.verification_uri_complete}`
          this.qrCodeRef.instance.style.display = "block"
          console.info(p.verification_uri_complete)
        }
      }, this.cancelSource.token).catch(e => console.error("Failed to sign in!", e))
    }
  }
}
