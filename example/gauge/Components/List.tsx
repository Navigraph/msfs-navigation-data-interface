/* eslint-disable @typescript-eslint/no-redundant-type-constituents */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-explicit-any */

/*
  Mostly taken directly from https://github.com/microsoft/msfs-avionics-mirror/blob/main/src/workingtitle-instruments-g1000/html_ui/Pages/VCockpit/Instruments/NavSystems/WTG1000/Shared/UI/List.tsx
  I'm not reinventing the wheel.
*/

import {
  ComponentProps,
  DisplayComponent,
  FSComponent,
  SubscribableArray,
  SubscribableArrayEventType,
  VNode,
} from "@microsoft/msfs-sdk";

/** The properties for the List component. */
interface ListProps extends ComponentProps {
  /**
   * The data for this list.
   * @type {any[]}
   */
  data: SubscribableArray<any>;

  /** A function defining how to render each list item. */
  renderItem: (data: any, index: number) => VNode;

  /** CSS class(es) to add to the root of the list component. */
  class?: string;
}

/** The List component. */
export class List extends DisplayComponent<ListProps> {
  private readonly _listContainer = FSComponent.createRef<HTMLElement>();

  /** @inheritdoc */
  public onAfterRender(): void {
    this.renderList();
    this.props.data.sub(this.onDataChanged.bind(this));
  }

  /**
   * A callback fired when the array subject data changes.
   * @param index The index of the change.
   * @param type The type of change.
   * @param item The item that was changed.
   */
  private onDataChanged(index: number, type: SubscribableArrayEventType, item: any | any[]): void {
    switch (type) {
      case SubscribableArrayEventType.Added:
        {
          const el = this._listContainer.instance.children.item(index);
          if (Array.isArray(item)) {
            for (let i = 0; i < item.length; i++) {
              this.addDomNode(item[i], index + i, el);
            }
          } else {
            this.addDomNode(item, index, el);
          }
        }
        break;
      case SubscribableArrayEventType.Removed:
        {
          if (Array.isArray(item)) {
            for (let i = 0; i < item.length; i++) {
              this.removeDomNode(index);
            }
          } else {
            this.removeDomNode(index);
          }
        }
        break;
      case SubscribableArrayEventType.Cleared:
        this._listContainer.instance.innerHTML = "";
        break;
    }
  }

  /**
   * Removes a dom node from the collection at the specified index.
   * @param index The index to remove.
   */
  private removeDomNode(index: number): void {
    const child = this._listContainer.instance.childNodes.item(index);
    this._listContainer.instance.removeChild(child);
  }

  /**
   * Adds a list rendered dom node to the collection.
   * @param item Item to render and add.
   * @param index The index to add at.
   * @param el The element to add to.
   */
  private addDomNode(item: any, index: number, el: Element | null): void {
    const node = this.renderListItem(item, index);
    if (el !== null) {
      node && el && FSComponent.renderBefore(node, el as any);
    } else {
      el = this._listContainer.instance;
      node && el && FSComponent.render(node, el as any);
    }
  }

  /**
   * Renders a list item
   * @param dataItem The data item to render.
   * @param index The index to render at.
   * @returns list item vnode
   * @throws error when the resulting vnode is not a scrollable control
   */
  private renderListItem(dataItem: any, index: number): VNode {
    return this.props.renderItem(dataItem, index);
  }
  /** Renders the list of data items. */
  private renderList(): void {
    // clear all items
    this._listContainer.instance.textContent = "";

    // render items
    const dataLen = this.props.data.length;
    for (let i = 0; i < dataLen; i++) {
      const vnode = this.renderListItem(this.props.data.get(i), i);
      if (vnode !== undefined) {
        FSComponent.render(vnode, this._listContainer.instance);
      }
    }
  }

  /** @inheritdoc */
  render(): VNode {
    return <div class={this.props.class ?? ""} ref={this._listContainer}></div>;
  }
}
