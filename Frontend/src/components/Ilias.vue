<template>
  <h1 class="text-5xl m-5">Better Ilias</h1>
  <div
    v-for="(child, index) in root_node.children"
    :key="index"
    class="ml-5 cursor-pointer"
  >
    <Folder
      :index="index"
      @set_invisible="handle_set_inivisible"
      @set_visible="handle_set_visible"
      :node="child"
    ></Folder>
  </div>

  <div class="right-0 top-0 fixed">
    <UpdateIcon :updating="updating" @click="update"></UpdateIcon>
  </div>
</template>

<script>
import { defineComponent, ref } from "vue";
import axios from "axios";
import { useVisibility } from "./compositions";

export default defineComponent({
  name: "Ilias",
  emits: ["new_note", "login_pls"],
  async setup(props, { emit }) {
    let resp = await axios.get("/api/node");
    let root_node = ref(resp.data);

    function handle_set_visible(path) {
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
        node.visible = true;
      }
      node.visible = true;
    }

    function handle_set_inivisible(path) {
      let node = root_node.value;
      for (let index of path.reverse()) {
        node = node.children[index];
      }
      node.visible = false;
    }

    let { edit_visibility } = useVisibility();

    let updating = ref(false);

    const update = () => {
      let start = Date.now();
      if (!updating.value) {
        updating.value = true;
        axios
          .get("api/update")
          .then((resp) => {
            if (resp.data.status == "set_token") {
              emit("login_pls");
              return;
            }
            root_node.value = resp.data.node;
            console.log("updated after", Date.now() - start, "ms");
          })
          .catch((e) => {
            console.error(e);
          })
          .finally(() => {
            updating.value = false;
          });
      }
    };

    update();

    return {
      root_node,
      handle_set_inivisible,
      handle_set_visible,
      edit_visibility,
      updating,
      update,
    };
  },
});
</script>
