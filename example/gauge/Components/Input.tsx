import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  Subscribable,
  SubscribableUtils,
  UUID,
  VNode,
} from "@microsoft/msfs-sdk";
import { InterfaceNavbarItemV2 } from "./Utils";

interface InputProps extends ComponentProps {
  value: Subscribable<string>;
  setValue: (value: string) => void;
  default?: Subscribable<string> | string;
  class?: string | Subscribable<string>;
  textarea?: boolean;
}

export class Input extends DisplayComponent<InputProps> {
  private readonly inputId = UUID.GenerateUuid();
  private readonly inputRef = FSComponent.createRef<HTMLInputElement>();

  onAfterRender(node: VNode): void {
    super.onAfterRender(node);

    this.props.value.map(val => (this.inputRef.instance.value = val));
    SubscribableUtils.toSubscribable(this.props.default ?? "", true).map(val => {
      this.inputRef.instance.placeholder = val;
    });

    this.inputRef.instance.addEventListener("input", () => this.props.setValue(this.inputRef.instance.value ?? ""));

    this.inputRef.instance.onfocus = this.onInputFocus;
    this.inputRef.instance.onblur = this.onInputBlur;
  }

  private getInputProps() {
    return { value: this.props.value, class: this.props.class };
  }

  /**
   * Method to handle when input focus is set
   * @param e The focus event.
   */
  private onInputFocus = (e: FocusEvent): void => {
    e.preventDefault();

    Coherent.trigger("FOCUS_INPUT_FIELD", this.inputId, "", "", this.inputRef.instance.value, false);
    Coherent.on("mousePressOutsideView", () => this.inputRef.instance.blur());
  };

  /**
   * Method to handle on input blur
   */
  private onInputBlur = (): void => {
    Coherent.trigger("UNFOCUS_INPUT_FIELD", "");
    Coherent.off("mousePressOutsideView");
  };

  render() {
    if (this.props.textarea)
      return (
        <textarea style="width:350px;height:100px;" ref={this.inputRef} {...this.getInputProps()}>
          {this.props.value}
        </textarea>
      );
    return <input ref={this.inputRef} {...this.getInputProps()} />;
  }
}

interface CheckboxProps extends ComponentProps {
  value: Subscribable<string>;
  setValue: (value: string) => void;
  default?: Subscribable<string> | string;
  class?: string;
}

export class Checkbox extends DisplayComponent<CheckboxProps> {
  private readonly isActive = this.props.value.map(val => (val == "true" ? true : false));

  private onClick = () => {
    this.props.setValue(this.isActive.get() ? "false" : "true");
  };

  render(): VNode {
    return (
      <InterfaceNavbarItemV2
        content={""}
        active={this.isActive}
        class={`h-full flex-grow bg-white text-black flex items-center justify-center hover:bg-gray-400 ${
          this.props.class ?? ""
        }`}
        activeClass="hover:!bg-green-700 !bg-green-500 !text-white"
        setActive={() => this.onClick()}
      >
        <span class="text-4xl">{this.isActive.map(val => (val ? "✔" : "X"))}</span>
      </InterfaceNavbarItemV2>
    );
  }
}
