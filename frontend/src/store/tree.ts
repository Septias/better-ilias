import { acceptHMRUpdate, defineStore } from 'pinia'
import { Api } from '~/api'

export const useTreeStore = defineStore('tree', () => {
  const root_node = ref()
  const api = new Api()
  const has_data = computed(() => root_node.value !== undefined)

  api.add_update_listener((new_data) => {
    root_node.value = new_data
  })

  function update() {
    api.update()
  }

  return {
    root_node,
    has_data,
    update,
  }
})

if (import.meta.hot)
  import.meta.hot.accept(acceptHMRUpdate(useTreeStore, import.meta.hot))
