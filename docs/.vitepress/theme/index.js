import DefaultTheme from 'vitepress/theme'
import HomeLayout from './HomeLayout.vue'
import './index.css'

export default {
  extends: DefaultTheme,
  // Use the Layout property to override the default wrapper
  Layout: HomeLayout 
}