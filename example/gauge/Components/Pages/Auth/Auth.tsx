import { ComponentProps, DisplayComponent, FSComponent, VNode } from "@microsoft/msfs-sdk";
import {
  DownloadProgressPhase,
  NavigationDataStatus,
  NavigraphEventType,
  NavigraphNavigationDataInterface,
} from "@navigraph/msfs-navigation-data-interface";
import { CancelToken } from "navigraph/auth";
import { packages } from "../../../Lib/navigraph";
import { AuthService } from "../../../Services/AuthService";
import { Dropdown } from "../../Dropdown";

interface AuthPageProps extends ComponentProps {
  setDatabaseInfo: (value: NavigationDataStatus) => void;
  navigationDataInterface: NavigraphNavigationDataInterface;
}

export class AuthPage extends DisplayComponent<AuthPageProps> {
  private readonly textRef = FSComponent.createRef<HTMLDivElement>();
  private readonly loginButtonRef = FSComponent.createRef<HTMLButtonElement>();
  private readonly navigationDataTextRef = FSComponent.createRef<HTMLDivElement>();
  private readonly qrCodeRef = FSComponent.createRef<HTMLImageElement>();
  private readonly dropdownRef = FSComponent.createRef<Dropdown>();
  private readonly downloadButtonRef = FSComponent.createRef<HTMLButtonElement>();

  private cancelSource = CancelToken.source();

  constructor(props: AuthPageProps) {
    super(props);

    this.props.navigationDataInterface.onEvent(NavigraphEventType.DownloadProgress, data => {
      switch (data.phase) {
        case DownloadProgressPhase.Downloading:
          this.displayMessage("Downloading navigation data...");
          break;
        case DownloadProgressPhase.Cleaning:
          if (!data.deleted) return;
          this.displayMessage(`Cleaning destination directory. ${data.deleted} files deleted so far`);
          break;
        case DownloadProgressPhase.Extracting: {
          // Ensure non-null
          if (!data.unzipped || !data.total_to_unzip) return;
          const percent = Math.round((data.unzipped / data.total_to_unzip) * 100);
          this.displayMessage(`Unzipping files... ${percent}% complete`);
          break;
        }
      }
    });
  }

  onAfterRender(node: VNode): void {
    super.onAfterRender(node);

    this.loginButtonRef.instance.addEventListener("click", () => this.handleClick());
    this.downloadButtonRef.instance.addEventListener("click", () => this.handleDownloadClick());

    AuthService.user.sub(user => {
      if (user) {
        this.qrCodeRef.instance.src = "";
        this.qrCodeRef.instance.style.display = "none";
        this.loginButtonRef.instance.textContent = "Log out";
        this.textRef.instance.textContent = `Welcome, ${user.preferred_username}`;
        this.displayMessage("");

        this.handleLogin();
      } else {
        this.loginButtonRef.instance.textContent = "Sign in";
        this.textRef.instance.textContent = "Not logged in";
      }
    }, true);
  }

  private async handleClick() {
    try {
      if (AuthService.getUser()) {
        await AuthService.signOut();
      } else {
        this.cancelSource = CancelToken.source(); // Reset any previous cancellations
        this.displayMessage("Authenticating.. Scan code (or click it) to sign in");
        await AuthService.signIn(p => {
          if (p) {
            this.qrCodeRef.instance.src = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${p.verification_uri_complete}`;
            this.qrCodeRef.instance.style.display = "block";
            this.qrCodeRef.instance.onclick = () => {
              OpenBrowser(p.verification_uri_complete);
            };
          }
        }, this.cancelSource.token);
      }
    } catch (err) {
      this.qrCodeRef.instance.style.display = "none";
      if (err instanceof Error) this.displayError(err.message);
      else this.displayError(`Unknown error: ${String(err)}`);
    }
  }

  private handleLogin() {
    // Let's display all of our packages
    packages
      .listPackages()
      .then(pkgs => {
        for (const pkg of pkgs) {
          this.dropdownRef.instance.addDropdownItem(pkg.format, pkg.format);
        }
      })
      .catch(e => console.error(e));
  }

  private async handleDownloadClick() {
    try {
      if (!this.props.navigationDataInterface.getIsInitialized())
        throw new Error("Navigation data interface not initialized");

      const format = this.dropdownRef.instance.getNavigationDataFormat();
      if (!format) throw new Error("Unable to fetch package: No navigation data format has been selected");

      // Get default package for client
      const pkg = await packages.getPackage(format);

      // Download navigation data to work dir and set active
      await this.props.navigationDataInterface.download_navigation_data(pkg.file.url);

      // Update navigation data status
      this.props.setDatabaseInfo(await this.props.navigationDataInterface.get_navigation_data_install_status());

      this.displayMessage("Navigation data downloaded");
    } catch (err) {
      if (err instanceof Error) this.displayError(err.message);
      else this.displayError(`Unknown error: ${String(err)}`);
    }
  }

  private displayMessage(message: string) {
    this.navigationDataTextRef.instance.textContent = message;
    this.navigationDataTextRef.instance.style.color = "white";
  }

  private displayError(error: string) {
    this.navigationDataTextRef.instance.textContent = error;
    this.navigationDataTextRef.instance.style.color = "red";
  }

  render(): VNode {
    return (
      <div class="size-full flex flex-col">
        <p class="mb-8 text-4xl">Authentication</p>
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
          </div>
        </div>
      </div>
    );
  }
}
