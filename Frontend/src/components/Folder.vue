<template >
  <svg
    focusable="false"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    class="text-accent hover:text-white fill-current inline"
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
    class="p-1 rounded-sm hover:bg-accent bg-opacity-25"
    @click="expanded = !expanded"
    >{{ node.title }}</span
  >
  <ul class="node_tree" :class="{ shrinked: !expanded }">
    <li
      v-for="(child, index) in node.children"
      :key="child.id"
      :class="[child.breed, 'node_tree_item']"
      :index="index"
    >
      <component
        :index="index"
        :is="child.breed"
        :node="child"
        @set_invisible="handle_set_inivisible"
        @set_visible="handle_set_visible"
      ></component>
    </li>
  </ul>
</template>

<script lang="ts">
import { ref, defineComponent } from "vue";
import File from "./File.vue";
import DirectLink from "./DirectLink.vue";
import Forum from "./Forum.vue";

export default defineComponent({
  components: { File, DirectLink, Forum },
  name: "Folder",
  emits: ["set_invisible", "set_visible"],
  props: {
    node: Object,
    index: Number,
  },
  methods: {
    handle_set_inivisible(path: Array) {
      path.push(this.index);
      this.$emit("set_invisible", path);
    },
    handle_set_visible(path: Array) {
      path.push(this.index);
      this.$emit("set_visible", path);
    },
  },
  setup() {
    let expanded = ref(true);
    return { expanded };
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
