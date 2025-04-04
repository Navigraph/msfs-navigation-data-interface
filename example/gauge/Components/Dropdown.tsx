import { ComponentProps, DisplayComponent, FSComponent, Subject, VNode } from "@microsoft/msfs-sdk";
import "./Dropdown.css";

export class Dropdown extends DisplayComponent<ComponentProps> {
  private readonly dropdownButtonRef = FSComponent.createRef<HTMLDivElement>();
  private readonly dropdownMenuRef = FSComponent.createRef<HTMLUListElement>();

  private readonly dropdownButtonText = Subject.create<string>("Select an item");

  private navigationDataFormat: null | string = null;

  public render(): VNode {
    return (
      <div class="dropdown">
        <div ref={this.dropdownButtonRef} class="dropdown-toggle">
          {this.dropdownButtonText}
        </div>
        <ul ref={this.dropdownMenuRef} class="dropdown-menu" />
      </div>
    );
  }

  public onAfterRender(node: VNode): void {
    super.onAfterRender(node);

    const dropdownButton = this.dropdownButtonRef.instance;
    const dropdownMenu = this.dropdownMenuRef.instance;

    dropdownButton.addEventListener("click", function () {
      dropdownMenu.style.display = dropdownMenu.style.display === "block" ? "none" : "block";
    });

    // Close the dropdown when clicking outside of it
    document.addEventListener("click", this.onDropdownItemClick.bind(this));
  }

  public onDropdownItemClick(event: Event): void {
    const dropdownButton = this.dropdownButtonRef.instance;
    const dropdownMenu = this.dropdownMenuRef.instance;

    const target = event.target as HTMLElement;
    if (!target) {
      return;
    }
    if (!dropdownMenu.contains(target) && !dropdownButton.contains(target)) {
      dropdownMenu.style.display = "none";
    } else if (dropdownMenu.contains(target)) {
      this.dropdownButtonText.set(target.textContent!);
      const navigationDataFormat = target.dataset.navigationDataFormat;
      if (navigationDataFormat) {
        this.navigationDataFormat = navigationDataFormat;
      }
    }
  }

  public addDropdownItem(text: string, format: string): void {
    const dropdownItem = document.createElement("li");
    dropdownItem.textContent = text;
    dropdownItem.dataset.navigationDataFormat = format;
    this.dropdownMenuRef.instance.appendChild(dropdownItem);
  }

  public getNavigationDataFormat(): string | null {
    return this.navigationDataFormat;
  }
}
