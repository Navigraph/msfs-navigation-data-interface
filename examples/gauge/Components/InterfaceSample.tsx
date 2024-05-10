import {
  ComponentProps,
  DisplayComponent,
  EventBus,
  FSComponent,
  MappedSubject,
  Subject,
  VNode,
} from "@microsoft/msfs-sdk"
import {
  DownloadProgressPhase,
  NavigraphEventType,
  NavigraphNavigationDataInterface,
} from "@navigraph/msfs-navigation-data-interface"
import { NavigationDataStatus } from "@navigraph/msfs-navigation-data-interface/types/meta"
import { CancelToken } from "navigraph/auth"
import { packages } from "../Lib/navigraph"
import { AuthService } from "../Services/AuthService"
import { Dropdown } from "./Dropdown"
import { Input } from "./Input"
import "./InterfaceSample.css"

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
  private readonly icaoInputRef = FSComponent.createRef<Input>()
  private readonly executeIcaoButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly sqlInputRef = FSComponent.createRef<Input>()
  private readonly executeSqlButtonRef = FSComponent.createRef<HTMLButtonElement>()
  private readonly outputRef = FSComponent.createRef<HTMLPreElement>()

  private readonly navigationDataStatus = Subject.create<NavigationDataStatus | null>(null)

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

  public renderDatabaseStatus(): VNode | void {
    return (
      <>
        <div
          class={MappedSubject.create(([status]) => {
            return status ? "vertical" : "hidden"
          }, this.navigationDataStatus)}
        >
          <div>{this.navigationDataStatus.map(s => `Install method: ${s?.status}`)}</div>
          <div>
            {this.navigationDataStatus.map(
              s => `Installed format: ${s?.installedFormat} revision ${s?.installedRevision}`,
            )}
          </div>
          <div>{this.navigationDataStatus.map(s => `Installed path: ${s?.installedPath}`)}</div>
          <div>{this.navigationDataStatus.map(s => `Installed cycle: ${s?.installedCycle}`)}</div>
          <div>{this.navigationDataStatus.map(s => `Latest cycle: ${s?.latestCycle}`)}</div>
          <div>{this.navigationDataStatus.map(s => `Validity period: ${s?.validityPeriod}`)}</div>
        </div>
        <div class={this.navigationDataStatus.map(status => (status ? "hidden" : "visible"))}>Loading status...</div>
      </>
    )
  }

  public render(): VNode {
    return (
      <div class="auth-container">
        <div class="horizontal">
          <div class="vertical">
            <h4>Step 1 - Sign in</h4>
            <div ref={this.textRef}>Loading</div>
            <div ref={this.loginButtonRef} class="button" />
            <div ref={this.navigationDataTextRef} />
            <img ref={this.qrCodeRef} class="qr-code" />
          </div>
          <div class="vertical">
            <h4>Step 2 - Select Database</h4>
            <Dropdown ref={this.dropdownRef} />
            <div ref={this.downloadButtonRef} class="button">
              Download
            </div>
            {this.renderDatabaseStatus()}
          </div>
        </div>

        <h4 style="text-align: center;">Step 3 - Query the database</h4>
        <div class="horizontal">
          <div class="vertical">
            <Input ref={this.icaoInputRef} value="ESSA" class="text-field" />
            <div ref={this.executeIcaoButtonRef} class="button">
              Fetch Airport
            </div>
            <div style="height:30px;"></div>
            <Input
              ref={this.sqlInputRef}
              textarea
              value="SELECT airport_name FROM tbl_airports WHERE airport_identifier = 'ESSA'"
              class="text-field"
            />
            <div ref={this.executeSqlButtonRef} class="button">
              Execute SQL
            </div>
          </div>
          <pre ref={this.outputRef} id="output">
            The output of the query will show up here
          </pre>
        </div>
      </div>
    )
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node)

    this.loginButtonRef.instance.addEventListener("click", () => this.handleClick())
    this.downloadButtonRef.instance.addEventListener("click", () => this.handleDownloadClick())

    // Populate status when ready
    this.navigationDataInterface.onReady(() => {
      this.navigationDataInterface
        .get_navigation_data_install_status()
        .then(status => this.navigationDataStatus.set(status))
        .catch(e => console.error(e))
    })

    this.executeIcaoButtonRef.instance.addEventListener("click", () => {
      console.time("query")
      this.navigationDataInterface
        .get_airport(this.icaoInputRef.instance.value)
        .then(airport => {
          console.info(airport)
          this.outputRef.instance.textContent = JSON.stringify(airport, null, 2)
        })
        .catch(e => console.error(e))
        .finally(() => console.timeEnd("query"))
    })

    this.executeSqlButtonRef.instance.addEventListener("click", () => {
      console.time("query")
      this.navigationDataInterface
        .execute_sql(this.sqlInputRef.instance.value, [])
        .then(result => {
          console.info(result)
          this.outputRef.instance.textContent = JSON.stringify(result, null, 2)
        })
        .catch(e => console.error(e))
        .finally(() => console.timeEnd("query"))
    })

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = ""
        this.qrCodeRef.instance.style.display = "none"
        this.loginButtonRef.instance.textContent = "Log out"
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`
        this.displayMessage("")

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
        this.displayMessage("Authenticating.. Scan code (or click it) to sign in")
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
      this.qrCodeRef.instance.style.display = "none"
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

      // Update navigation data status
      this.navigationDataInterface
        .get_navigation_data_install_status()
        .then(status => this.navigationDataStatus.set(status))
        .catch(e => console.error(e))

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
