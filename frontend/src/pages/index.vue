<script setup lang="ts" async>
import { computed, ref } from 'vue'
import NProgress from 'nprogress'
import { invoke } from '@tauri-apps/api'
import type { IlNode } from '~/types'
import { IlNodeType } from '~/types'
import { get_breed, invoke_log } from '~/utils'
const root_node = ref(await invoke_log('get_root') as IlNode)

const router = useRouter()
invoke('login_cached').catch(() => router.push('/login'))

function handle_set_visible(path: any) {
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

async function update() {
  NProgress.start()
  await invoke_log('update_root')
  root_node.value = await invoke_log('get_root') as IlNode
  NProgress.done()
}
const folders = computed(() => root_node.value.children!.filter(node => get_breed(node.breed) === IlNodeType.Folder))
</script>

<template lang="pug">
.right-0.top-0.fixed.p-2
  button.i-carbon-download.text-white(@click='update')
.flex.justify-center.items-center.flex-col
  div.flex.flex-col.gap
    h1.text-5xl.m-5.text-white Better Ilias
    .ml-5.cursor-pointer(v-for='(child, index) in folders' :key='index')
      folder(:index='index' :node='child' @set_invisible='handle_set_inivisible' @set_visible='handle_set_visible')
</template>

<style lang="sass">
.gap
  gap: 0.2rem
</style>
