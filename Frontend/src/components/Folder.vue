<template >
  <svg
    v-if="node.children"
    focusable="false"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    class="text-accent hover:text-white fill-current inline"
  >
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
  </svg>

  <span
    class="p-1 rounded-sm hover:bg-accent bg-opacity-25"
    @click="expanded = !expanded"
    >{{ node.title }}</span
  >
  <ul class="node_tree" :class="{ shrinked: !expanded }">
    <li
      v-for="child in node.children"
      :key="child.id"
      :class="[child.breed, 'node_tree_item']"
    >
      <component :is="child.breed" :node="child"></component>
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
  props: {
    node: Object,
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
