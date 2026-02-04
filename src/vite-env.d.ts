/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

declare module "sortablejs" {
  interface SortableOptions {
    animation?: number;
    handle?: string;
    forceFallback?: boolean;
    fallbackOnBody?: boolean;
    ghostClass?: string;
    chosenClass?: string;
    dragClass?: string;
    onEnd?: (ev: { oldIndex: number; newIndex: number }) => void;
  }
  interface SortableInstance {
    destroy(): void;
  }
  export function create(el: HTMLElement, options: SortableOptions): SortableInstance;
}
