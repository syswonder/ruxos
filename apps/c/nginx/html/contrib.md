# 如何用git协作

假定:

- 你的github账号为 panda

- 要贡献的上游仓库为 https://github.com/syswonder/syswonder-web.git

- 你的个人主库为 https://github.com/panda/syswonder-web

**1. Fork repo**

在浏览器中打开上游主库 https://github.com/syswonder/syswonder-web

点击右上角的"Fork" 按钮

**2. Clone repo**

将Fork得到的个人主库 clone到本地

```bash
git clone https://github.com/panda/syswonder-web
```

!> 请记得将panda替换为您自己的用户名


```bash
# 设置 upstream
git remote add upstream https://github.com/syswonder/syswonder-web.git
# 禁止直接向 upstream 推送代码
git remote set-url --push upstream no_push
```

**3. work, commit & push**

多人协作不要在主分支main上工作，要另建工作分支。

为本次工作创建一个分支，命名为 `dev`

```bash
git checkout -b dev
```

此时就可以在该分支下工作了, 不断修改代码，不断 commit

```bash
git add .

git commit -m "your commit message"
```

如果经过测试，觉得在本地完成了代码工作，就可以将代码推送到自己的个人主库
中了。

```bash
git push origin dev
```

push到自己的个人主库中后，就可以准备创建`Pull Request`了。

**4. pull request**

为防冲突，先同步upstream

```bash
git fetch upstream
git merge upstream/main
```

此时请确保本地位于工作分支。如果合并过程有冲突，要负责解决冲突，并提交。

将消解了冲突的最新本地代码，推送到个人主库的dev分支。

```bash
git push origin dev
```

到自己个人主库的首页, 准备 **Pull Request** , 

base repo 选择 syswonder/syswonder-web main

head repo 选择 panda/syswonder-web dev

提交，等待上游库管理员审核。

**5. sync origin main**

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
git checkout dev
```
