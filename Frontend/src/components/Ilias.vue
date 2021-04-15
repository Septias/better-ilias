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
  <div
    class="fixed h-full w-full flex justify-center items-center top-0 bg-main bg-opacity-50"
    @click="disable_login"
    v-show="login"
  >
    <form
      class="bg-main rounded-xl border-2 border-accent p-4 custom_form text-xl"
      @submit.prevent="send_credentials"
      @click.stop=""
    >
      <p v-if="wrong" class="text-sm text-accent">{{ wrong }}</p>
      <label>Benutzername</label>
      <input v-model="username" autocomplete="username" class="block w-full" />
      <label>Passwort</label>
      <input
        v-model="password"
        autocomplete="current-password"
        class="block w-full"
        type="password"
      />
      <input class="mr-1" type="checkbox" v-model="persistent" />
      <p class="inline-block text-sm">Angemeldet bleiben</p>
      <button
        type="submit"
        class="button px-2 rounded float-right"
        :class="requesting ? 'bg-gray-600' : 'bg-accent'"
        :disabled="requesting"
      >
        Ok!
      </button>
    </form>
  </div>
</template>

<style lang="sass" scoped>
.custom_form
  label
    @apply text-accent

  input
    @apply bg-light-main
    @apply p-1
    @apply mb-3
</style>

<script>
import { defineComponent, ref } from "vue";
import axios from "axios";
import { useVisibility } from "./compositions";

export default defineComponent({
  name: "Ilias",
  emits: ["new_note"],
  async setup() {
    let resp = await axios.get("/api/node");
    let data = resp.data;
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
            updating.value = false;
            if (resp.data.status == "set_token") {
              login.value = true;
              return;
            }
            root_node.value = resp.data.node;
            console.log("updated after", Date.now() - start, "ms");
          })
          .catch((e) => {
            console.error(e);
            updating.value = false;
          });
      }
    };

    //update();

    const login = ref(false);
    const username = ref("");
    const password = ref("");
    const persistent = ref(false);
    const wrong = ref("");
    const requesting = ref(false);
    const send_credentials = () => {
      requesting.value = true;
      wrong.value = "";
      axios
        .post("api/credentials", {
          username: username.value,
          password: password.value,
          persistent: persistent.value,
        })
        .then((resp) => {
          requesting.value = false;
          console.log(resp.data.status);
          if (resp.data.status == "ok") {
            wrong.value = "";
            login.value = false;
            update();
          } else {
            wrong.value = resp.data.status;
          }
        })
        .catch((err) => {
          console.err(err);
          requesting.value = false;
        });
    };

    const disable_login = () => {
      if (!requesting.value) {
        login.value = !login.value;
      }
    };

    return {
      root_node,
      handle_set_inivisible,
      handle_set_visible,
      edit_visibility,
      updating,
      update,
      login,
      username,
      password,
      send_credentials,
      persistent,
      wrong,
      requesting,
      disable_login,
    };
  },
});
</script>
