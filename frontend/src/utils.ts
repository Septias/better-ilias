export const get_breed = function (breed: any) {
  if (typeof breed == 'object') {
    return Object.keys(breed)[0]
  }
  else {
    return breed
  }
}
