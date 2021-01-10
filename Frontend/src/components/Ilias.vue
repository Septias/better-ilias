<template>
  <h1 class="text-5xl m-5">Better Ilias</h1>
  <div v-for="(child, index) in root_node.children" class="ml-5 cursor-pointer">
    <Folder
      :index="index"
      @set_invisible="handle_set_inivisible"
      :node="child"
    ></Folder>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import axios from "axios";

export default defineComponent({
  name: "Ilias",
  async setup() {
    let resp = await axios.get("/api/node");
    let data: IlNode = resp.data as IlNode;
    console.log(data);
    let root_node = ref(data);

    function handle_set_inivisible(path) {
      console.log(path);
      //console.log(root_node.value.children[path.pop()]);
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
      }
      node.visible = false;
    }

    return { root_node, handle_set_inivisible };
  },
});
</script>

<style>
.transparent {
  color: white;
}
</style>