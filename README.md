# cnblog
## 介绍

​	rust重写cnblog自动上传。期望允许上传和下载。思路：将cnblog作为中央存储器，类似笔记类软件的同步。

## 使用

使用说明：

![image-20220525203008533](https://nsfoxer-oss.oss-cn-beijing.aliyuncs.com/img/6a4a53ba8a476aa99968eec57ae43dd7.png)

```shell
-c: 指定config的存储文件夹路径，默认为家目录的~/.config/cnblog/
-h: 帮助说明
-r: 指定要上传博客所在的文件绝对路径（重要）
-V: 版本信息
```

## 原理

​	`cnblog`依赖博客园提供的`metaweblog`接口。将所有上传的博客信息数据存储在sqlite中。同时对sqlite进行base64编码，并上传至博客园。以此方式将博客园作为一个中心服务，实现博客的同步。

## 例子

### 简单使用	

所有markdown格式的博客都位于`~/Documents/articles`下，使用

`./cnblog -r ~/Documents/articles`将把该路径下的所有`md`结尾的文章上传至博客园。

`notes: 每次的'-r'指定的路径都必须是所有博客的“根路径”`

### 同步

​	所有新增的博客和有修改的博客将都被识别，进行上传。已删除的博客会被放置在“博客根路径”下的`.cnblog_deleted`文件夹下，并以博客id命名。

## 注意

​	目前`cnblog`没有经过详细的测试，存在不稳定风险。（放心，最坏的结果也不会完全删除你的博客）
