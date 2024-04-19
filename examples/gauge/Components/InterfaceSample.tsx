import { ComponentProps, DisplayComponent, EventBus, FSComponent, VNode } from "@microsoft/msfs-sdk"
import { CancelToken } from "navigraph/auth"
import { packages } from "../Lib/navigraph"
import { AuthService } from "../Services/AuthService"
import "./InterfaceSample.css"
import {
  DownloadProgressPhase,
  NavigraphEventType,
  NavigraphNavigationDataInterface,
} from "@navigraph/msfs-navigation-data-interface"
import { Dropdown } from "./Dropdown"

interface InterfaceSampleProps extends ComponentProps {
  bus: EventBus
}

export class InterfaceSample extends DisplayComponent<InterfaceSampleProps> {
  private readonly textRef = FSComponent.createRef<HTMLDivElement>()
  private readonly navigationDataTextRef = FSComponent.createRef<HTMLDivElement>()
  private readonly loginButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly qrCodeRef = FSComponent.createRef<HTMLImageElement>()
  private readonly dropdownRef = FSComponent.createRef<Dropdown>()
  private readonly downloadButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly executeButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly inputRef = FSComponent.createRef<HTMLInputElement>()

  private cancelSource = CancelToken.source()

  private navigationDataInterface: NavigraphNavigationDataInterface

  constructor(props: InterfaceSampleProps) {
    super(props)

    this.navigationDataInterface = new NavigraphNavigationDataInterface()

    this.navigationDataInterface.onEvent(NavigraphEventType.DownloadProgress, data => {
      switch (data.phase) {
        case DownloadProgressPhase.Downloading:
          this.displayMessage("Downloading navigation data...")
          break
        case DownloadProgressPhase.Cleaning:
          if (!data.deleted) return
          this.displayMessage(`Cleaning destination directory. ${data.deleted} files deleted so far`)
          break
        case DownloadProgressPhase.Extracting: {
          // Ensure non-null
          if (!data.unzipped || !data.total_to_unzip) return
          const percent = Math.round((data.unzipped / data.total_to_unzip) * 100)
          this.displayMessage(`Unzipping files... ${percent}% complete`)
          break
        }
      }
    })
  }

  public render(): VNode {
    return (
      <div class="auth-container">
        <div class="horizontal">
          <div class="vertical">
            <div ref={this.textRef}>Loading</div>
            <div ref={this.loginButtonRef} class="button" />
            <div ref={this.navigationDataTextRef} />
            <img ref={this.qrCodeRef} class="qr-code" />
          </div>
          <div class="vertical">
            <Dropdown ref={this.dropdownRef} />
            <div ref={this.downloadButtonRef} class="button">
              Download
            </div>
            <input ref={this.inputRef} type="text" id="sql" name="sql" value="ESSA" class="text-field" />
            <div ref={this.executeButtonRef} class="button">
              Fetch Airport
            </div>
          </div>
        </div>
      </div>
    )
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.loginButtonRef.instance.addEventListener("click", () => this.handleClick())
    this.downloadButtonRef.instance.addEventListener("click", () => this.handleDownloadClick())

    this.executeButtonRef.instance.addEventListener("click", () => {
      console.time("query")
      this.navigationDataInterface
        .get_airport(this.inputRef.instance.value)
        .then(airport => console.info(airport))
        .catch(e => console.error(e))
        .finally(() => console.timeEnd("query"))
    })

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = ""
        this.qrCodeRef.instance.style.display = "none"
        this.loginButtonRef.instance.textContent = "Log out"
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`

        this.handleLogin()
      } else {
        this.loginButtonRef.instance.textContent = "Sign in"
        this.textRef.instance.textContent = "Not logged in"
      }
    }, true)
  }

  private async handleClick() {
    try {
      if (AuthService.getUser()) {
        await AuthService.signOut()
      } else {
        this.cancelSource = CancelToken.source() // Reset any previous cancellations
        await AuthService.signIn(p => {
          if (p) {
            this.qrCodeRef.instance.src = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${p.verification_uri_complete}`
            this.qrCodeRef.instance.style.display = "block"
            this.qrCodeRef.instance.onclick = () => {
              OpenBrowser(p.verification_uri_complete)
            }
          }
        }, this.cancelSource.token)
      }
    } catch (err) {
      if (err instanceof Error) this.displayError(err.message)
      else this.displayError(`Unknown error: ${String(err)}`)
    }
  }

  private handleLogin() {
    // Let's display all of our packages
    packages
      .listPackages()
      .then(pkgs => {
        for (const pkg of pkgs) {
          this.dropdownRef.instance.addDropdownItem(pkg.format, pkg.format)
        }
      })
      .catch(e => console.error(e))
  }

  private async handleDownloadClick() {
    try {
      if (!this.navigationDataInterface.getIsInitialized()) throw new Error("Navigation data interface not initialized")

      const format = this.dropdownRef.instance.getNavigationDataFormat()
      if (!format) throw new Error("Unable to fetch package: No navigation data format has been selected")

      // Get default package for client
      const pkg = await packages.getPackage(format)

      // Download navigation data to work dir
      await this.navigationDataInterface.download_navigation_data(pkg.file.url)
      this.displayMessage("Navigation data downloaded")
    } catch (err) {
      if (err instanceof Error) this.displayError(err.message)
      else this.displayError(`Unknown error: ${String(err)}`)
    }
  }

  private displayMessage(message: string) {
    this.navigationDataTextRef.instance.textContent = message
    this.navigationDataTextRef.instance.style.color = "white"
  }

  private displayError(error: string) {
    this.navigationDataTextRef.instance.textContent = error
    this.navigationDataTextRef.instance.style.color = "red"
  }
}
