<script setup lang="ts" async>
import { invoke } from '@tauri-apps/api'
import { computed, ref } from 'vue'
import NProgress from 'nprogress'
import type { IlNode } from '~/types'
import { IlNodeType } from '~/types'
import { get_breed } from '~/utils'
const root_node = ref(await invoke('get_root') as IlNode)
console.log(root_node.value)

function handle_set_visible(path: any) {
  console.log(path)
  let node = root_node.value
  for (const index of path.reverse()) {
    node = node.children![index]
    node.visible = true
  }
  node.visible = true
}

function handle_set_inivisible(path: any) {
  let node = root_node.value
  for (const index of path.reverse()) {
    node = node.children![index]
  }
  node.visible = false
}

const updating = ref(false)
async function update() {
  NProgress.start()
  console.log(await invoke('update_root'))
  NProgress.done()
}
const folders = computed(() => root_node.value.children!.filter(node => get_breed(node.breed) === IlNodeType.Folder))
</script>

<template>
  <h1 class="text-5xl m-5" @click="update">
    Better Ilias
  </h1>
  <div
    v-for="(child, index) in folders"
    :key="index"
    class="ml-5 cursor-pointer"
  >
    <Folder
      :index="index"
      :node="child"
      @set_invisible="handle_set_inivisible"
      @set_visible="handle_set_visible"
    />
  </div>

  <div class="right-0 top-0 fixed">
    <UpdateIcon :updating="updating" @click="update" />
  </div>
</template>
