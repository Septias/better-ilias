<template >
  <span v-if="node.visible || edit_visibility">
    <div class="pluslogic">
      <svg
        focusable="false"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
        class="text-accent hover:text-white fill-current inline"
        @click="expanded = !expanded"
      >
        <template v-if="node.children.length">
          <path
            v-if="expanded"
            d="M5.843 9.593L11.5 15.25l5.657-5.657l-.707-.707l-4.95 4.95l-4.95-4.95l-.707.707z"
            fill="currentColor"
          ></path>
          <path
            v-else
            d="M8.593 18.157L14.25 12.5L8.593 6.843l-.707.707l4.95 4.95l-4.95 4.95l.707.707z"
            fill="currentColor"
          ></path>
        </template>
        <rect
          v-else
          x="9"
          y="9"
          width="5"
          height="5"
          style="fill: rgba(0, 0, 0, 0); stroke-width: 2; stroke: currentColor"
        />
      </svg>
      <span
        class="text-white select-none"
        :class="{ 'text-opacity-25': !node.visible && edit_visibility }"
        @click.exact="handle_click"
        @click.ctrl.exact="open_page"
        @click.shift.exact="open_folder"
      >
        <span class="p-1 rounded-sm hover:bg-accent">
          {{ node.title }}
        </span>
      </span>

      <bx-bx-edit
        class="plus text-accent"
        @click="
          () => {
            activate_note(node);
          }
        "
      />
    </div>

    <ul class="node_tree" :class="{ shrinked: !expanded }">
      <li
        v-for="(child, index) in node.children"
        :key="child.id"
        :class="[child.breed, 'node_tree_item']"
        :index="index"
      >
        <component
          :index="index"
          :is="get_type(child.breed)"
          :node="child"
          @set_invisible="handle_set_inivisible"
          @set_visible="handle_set_visible"
        ></component>
      </li>
    </ul>
  </span>
</template>

<style lang="sass" scoped>
.pluslogic:hover
  > .plus
    display: inline-block !important

.pluslogic
  > .plus
    display: none
</style>

<script lang="ts">
import { ref, defineComponent, PropType } from "vue";
import File from "./File.vue";
import DirectLink from "./DirectLink.vue";
import Forum from "./Forum.vue";
import { useNotes, useVisibility } from "./compositions";
import axios from "axios";
import { IlNode } from "./IlTypes";
import Video from "./Video.vue";
import Exercise from "./Exercise.vue";

export default defineComponent({
  components: { File, DirectLink, Forum, Video, Exercise },
  name: "Folder",
  emits: ["set_invisible", "set_visible", "new_note"],
  props: {
    node: {
      type: Object as PropType<IlNode>,
      required: true,
    },
    index: {
      type: Number,
      required: true,
    },
  },
  methods: {
    handle_set_inivisible(path: Array<Number>) {
      path.push(this.index);
      this.$emit("set_invisible", path);
    },
    handle_set_visible(path: Array<Number>) {
      path.push(this.index);
      this.$emit("set_visible", path);
    },
    open_page() {
      window.open("https://ilias.uni-freiburg.de/" + this.node.uri);
    },
  },
  setup(props, { emit }) {
    let expanded = ref(false);
    let { edit_visibility } = useVisibility();
    function handle_click() {
      if (edit_visibility.value) {
        if (props.node.visible) {
          emit("set_invisible", [props.index]);
        } else {
          emit("set_visible", [props.index]);
        }
      } else {
        expanded.value = !expanded.value;
      }
    }

    const { activate_note, reset_note, active } = useNotes();

    function open_folder() {
      axios.post("/api/open/", {
        path: props.node.breed.Folder.path,
      });
    }

    const get_type = function (breed: any) {
      if (typeof breed == "object") {
        return Object.keys(breed)[0];
      } else {
        return breed;
      }
    };
    return {
      expanded,
      edit_visibility,
      handle_click,
      get_type,
      activate_note,
      open_folder,
    };
  },
});
</script>

<style>
.node_tree {
  list-style: none;
  padding-inline-start: 20px;
  max-height: 100%;
}

.node_tree_item {
  cursor: pointer;
}

.shrinked {
  max-height: 0px;
  overflow: hidden;
}
</style>
