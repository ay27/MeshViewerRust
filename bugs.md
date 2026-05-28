发现的 bug：

1. 希望侧边栏是独立的 div（或 block），而不是 overlay 在 3D 场景上，现在 overlap 的模式，会导致场景的一部分被遮挡
2. 线框、网格、顶点、材质、法线几个模式切换时，配置清理不够干净。比如颜色，当从顶点切换至材质或者网格时，顶点的颜色会带过来，影响到了面片的颜色
3. 在系统的文件浏览器打开模型文件，自动拉起这个 app 时会报错：The document xxx can not be opened. Can not open files in the "FBX file" format.
4. 需要新增一个视图类型：部件视图，这个视图下，给每个独立的 mesh node 一个特定的颜色，以便于区分不同的 mesh。同时在右侧的 info 面板里，显示对应的颜色块，以便快速对应
