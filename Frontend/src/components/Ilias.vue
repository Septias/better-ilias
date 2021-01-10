<template>
  <h1 class="text-5xl m-5">Better Ilias</h1>
  <div v-for="(child, index) in root_node.children" class="ml-5 cursor-pointer">
    <Folder
      :index="index"
      @set_invisible="handle_set_inivisible"
      @set_visible="handle_set_visible"
      :node="child"
    ></Folder>
  </div>
  <div class="right-0 top-0 fixed">
    <div
      class="bg-light-main p-2 m-2 rounded cursor-pointer"
      @click="edit_visibility = !edit_visibility"
    >
      <svg width="1em" height="1em" viewBox="0 0 24 24">
        <path
          d="M11.5 18c3.989 0 7.458-2.224 9.235-5.5A10.498 10.498 0 0 0 11.5 7a10.498 10.498 0 0 0-9.235 5.5A10.498 10.498 0 0 0 11.5 18zm0-12a11.5 11.5 0 0 1 10.36 6.5A11.5 11.5 0 0 1 11.5 19a11.5 11.5 0 0 1-10.36-6.5A11.5 11.5 0 0 1 11.5 6zm0 2a4.5 4.5 0 1 1 0 9a4.5 4.5 0 0 1 0-9zm0 1a3.5 3.5 0 1 0 0 7a3.5 3.5 0 0 0 0-7z"
          fill="currentColor"
        ></path>
      </svg>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import axios from "axios";
import { useVisibility } from "./compositions";

export default defineComponent({
  name: "Ilias",
  async setup() {
    let resp = await axios.get("/api/node");
    let data: IlNode = resp.data as IlNode;
    console.log(data);
    let root_node = ref(data);

    function handle_set_visible(path) {
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
        node.visible = true;
      }
      node.visible = true;
    }

    function handle_set_inivisible(path) {
      //console.log(root_node.value.children[path.pop()]);
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
      }
      node.visible = false;
    }

    let { edit_visibility } = useVisibility();

    return {
      root_node,
      handle_set_inivisible,
      handle_set_visible,
      edit_visibility,
    };
  },
});
</script>

