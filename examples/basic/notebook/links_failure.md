---
date_created: 17-09-2021
---

# Observing mode transitions 
A condition under which we can observe sequences is that the system is $(N, \alpha, \omega)$-MO, such that $\alpha + \omega \leq N + 2$.
A helpful result would be the following.

**Question. ** given that a system is $(N, \alpha, \omega)$-MO, is it 
$(N+1, \alpha, \omega)$-MO?

## Simplest case: linear, autonomous

**To prove: ** 
if the system is $(N, \alpha, \omega)$-MO, i.e.,  
$$
\rank(\regmatrix{\obs(\theta) & \obs(\theta')}) = 2n
$$
for all $\theta, \theta' \in \W^{N+1}$ such that, $\theta_{\alpha:N-\omega} \neq \theta_{\alpha:N-\omega}'$, 
then, also 
$$
\rank(\regmatrix{\obs(\theta) & \obs(\theta')}) = 2n
$$
for all $\theta, \theta' \in \W^{N+2}$ such that $\theta_{\alpha:N+1-\omega} \neq \theta_{\alpha:N+1-\omega}'$.

This is not true. **Counterexample.** See [[Numerical experiments mode estimation switching systems#Example forward discernability Babaali and Egerstedt|example Babaali.]]

### Backward discernability 
This is fundamentally related to [[Backwards discernibility]]. If all the modes a backward discernable at index 1, then once we have identified one mode, we can identify all subsequent modes as well. (What is the condition for this to hold?)

We can somehow relax this, requiring that only once we have identified a subsequence, we can keep identifying it, but this requires backward discernability at some longer index. 
The combination we need then is is as follows: 
- System is $(N, \alpha, \omega)$-MO
- All modes are $(N-\alpha-\omega)$-BD 

### Sufficient conditions
If the system is $(1+N', 0, N')$-MO and $A_i$ are invertible, 
then, it is $(N+N', 0, N')$-MO, as shown in 
[[Babaali - Observability of Switched Linear Systems (2004)#Recursive mode estimation]].



## Controlled system
The counterexample mentioned above exploits the possibility of the controller 
to map to a state that lies in the set of conflict. However, if this set is empty, then this does not hold. 

### Extensions of proposition 4 in Babaali et al. (2004)

#### Case 1: from $N=1$ to any $N > 1$
We extend the statement of Proposition 4 to controlled systems. 

Recall the following definition.
<span class="definition" name="MO (controlled)">
We say that a controlled system is $(1+N', 0, N')$-MO if
for all paths $\theta, \theta' \in \W^{N'+1}$ with $\theta_0 \neq \theta_0'$,
$$
    \obs(\theta) x + G(\theta) u
    {}\nfd{}
    \obsaug(\theta, x, u)
     {}\neq{}
    \obsaug(\theta', x', u),
    \quad \forall x,x' \in \Re^{\ns}, 
$$
for almost every $u \in \Re^{\na N'}$. 
</span>

Furthermore, let us define the following:
$$
    \hat{c}(\theta, \theta')
    {}\dfn{}
    \{x \in \Re^{\ns} \mid \exists x' \in \Re^{\ns} : C_{\theta} x = C_{\theta'}x' \}.
$$
and
$$
    \begin{aligned}
      c(\theta, \theta', u)
      {}&\dfn{}
      \{
        x \in \Re^{\ns}
            \mid
        \obsaug(\theta, x, u) = \obsaug(\theta', x', u),\;
        x' \in \Re^{\ns}
      \}\\
      {}&={}
      \left\{
        x \in \Re^{\ns}
         \sep
        \begin{matrix}
            C_{\theta_0}x = C_{\theta_0}x',\quad x' \in \Re^{\ns},\\
            \obsaug(\seq{\theta}{1}{N'}, \bar{x}, \seq{u}{1}{N'-1})
            {}={}
            \obsaug(\seq{\theta}{1}{N'}, \bar{x}', \seq{u}{1}{N'-1}),\\
            \bar{x}  = A_{\theta_0}x,\;
            \bar{x}' = A_{\theta_0} x'
        \end{matrix}
    \right\}\\
    {}&\subseteq{}
    \hat{c}(\theta_{0}, \theta_{0}') \cap c(\seq{\theta}{1}{N'}, \seq{\theta'}{1}{N'}, \seq{u}{1}{N'-1}).
    \end{aligned}
$$
Clearly, the system is $(1+N', 0, N')$-MO iff $c(\theta, \theta', u) = \emptyset$ for almost every $u \in \Re^{\na N'}$.
The following trivial fact will help us prove the desired
results below.
<span class="lemma">
Let $\theta \bar{\theta}, \theta' \bar{\theta'} \in \W^{N+1}$ 
denote the concatenation of the paths $\theta, \theta' \in \W^{N}$ and 
the modes $\bar{\theta}, \bar{\theta'} \in \W$, respectively. 
It holds that 
$$
    c(\theta \bar{\theta}, \theta' \bar{\theta'}, u) \subseteq c(\theta, \theta', \seq{u}{0}{N-2})
$$
for all $u = (u_0, \dots, u_{N-1})\in \Re^{\na N}$.
</span>
<span class="proof">
The statement follows directly from the definition: 
$$
\begin{aligned}
      x \in c(\theta \bar{\theta}, \theta' \bar{\theta'}, u) \subseteq \Re^{\ns}
      &{}\Leftrightarrow{}
        \obsaug(\theta \bar{\theta}, x, u) = \obsaug(\theta' \bar{\theta'}, x', u)
        \text{ for some } 
        x' \in \Re^{\ns}.\\
    &{}\Leftrightarrow{}
    \begin{cases} 
        \obsaug(\theta, x, \seq{u}{0}{N-2}) = \obsaug(\theta', x', \seq{u}{0}{N-2})\\
        C_{\bar{\theta}} x_{N} = C_{\bar{\theta'}} x_{N}',
    \end{cases}
\end{aligned}
$$
with $x_{N}$ the state at time $N$, from initial condition $x$,
under mode sequence $\theta$ and controls $\seq{u}{0}{N-2}$, 
and $x_{N}'$ analogously defined with initial condition $x'$ 
and mode sequence $\theta'$.
On the other hand,
$$  
x \in c(\theta, \theta', \seq{u}{0}{N-2}) \Leftrightarrow \obsaug(\theta, x, \seq{u}{0}{N-2}) = \obsaug(\theta', x', \seq{u}{0}{N-2}) \text{ for some } x' \in \Re^{\ns}.
$$ 
Therefore, if $x \in c(\theta \bar{\theta}, \theta' \bar{\theta'}, u)$, then $x \in c(\theta, \theta', \seq{u}{0}{N-2})$, as required.
</span>
^lemma2
<span class="lemma">
Let $\bar{\theta}\theta, \bar{\theta'}\theta' \in \W^{N+1}$ denote the concatenations 
of $\bar{\theta}, \bar{\theta'} \in \W$ and $\theta, \theta' \in \W^{N}$, respectively.
Then,
$$
    c(\bar{\theta}\theta,  \bar{\theta'}\theta', u) \subseteq T_{\bar{\theta}, u_0}^{-1}c(\theta, \theta', \seq{u}{1}{N-1}), 
$$
for any $u = (u_0, \dots, u_{N-1}) \in \Re^{\na N}$, where $T_{\bar{\theta} u_0}^{-1}(C)$ denotes the pre-image of $C$ under the affine map $T_{\bar{\theta} u_0}: x \mapsto A_{\bar{\theta}}x + B_{\bar{\theta}}u_{0}$.
</span>
<span class="proof">
This again follows directly from the definition:
$$
\begin{aligned}
      c(\bar{\theta} \theta, \bar{\theta'} \theta', u)
    &{}={}
        \left \{x \in \Re^{\ns} \sep
        \begin{matrix}
            C_{\bar{\theta}} x = C_{\bar{\theta'}} x', \; x' \in \Re^{\ns},\\
            \obsaug(\theta, \bar{x}, \seq{u}{1}{N-1}) = \obsaug(\theta', \bar{x'}, \seq{u}{1}{N-1}),\\
            \bar{x} = A_{\bar{\theta}}x + B_{\bar{\theta}}u_0, \quad \bar{x'} = A_{\bar{\theta'}}x' + B_{\bar{\theta'}}u_0
        \end{matrix}
        \right \}\\
    &{}\subseteq{}
        \left \{x \in \Re^{\ns} \sep
        \begin{matrix}
            \obsaug(\theta, \bar{x}, \seq{u}{1}{N-1}) = \obsaug(\theta', \bar{x'}, \seq{u}{1}{N-1})\\
            \bar{x} = A_{\bar{\theta}}x + B_{\bar{\theta}} u_0, \quad \bar{x'} \in \range(A_{\bar{\theta'}}) + B_{\bar{\theta'}}u_0 
        \end{matrix}
        \right \} \\ 
    &{}\subseteq{}
        \left \{x \in \Re^{\ns} \sep
        \begin{matrix}
            \obsaug(\theta, \bar{x}, \seq{u}{1}{N-1}) = \obsaug(\theta', \bar{x'}, \seq{u}{1}{N-1})\\
            \bar{x} - B_{\bar{\theta}}u_0 = A_{\bar{\theta}}x, \quad \bar{x'} \in \Re^{\ns}
        \end{matrix}
        \right \} \\ 
    &{}={}
        A_{\bar{\theta}}^{-1}
        \left( 
        \left \{\bar{x} \in \Re^{\ns} \sep
        \begin{matrix}
            \obsaug(\theta, \bar{x}, \seq{u}{1}{N-1}) = \obsaug(\theta', \bar{x'}, \seq{u}{1}{N-1})\\
            \bar{x'} \in \Re^{\ns}
        \end{matrix}
        \right \}
        - B_{\bar{\theta}}u_0 \right ) \\ 
    &{}={}
        A_{\bar{\theta}}^{-1}
        (c(\theta, \theta', \seq{u}{1}{N-1}) - B_{\bar{\theta}}u_0)\\ 
    &{}={}
        T_{\bar{\theta}, u_0}^{-1}c(\theta, \theta', \seq{u}{1}{N-1}).
\end{aligned}
$$
&#8718;<br>
</span>
Based on these facts, we can now state the following result.
#### From $N$ to $N+1$
We should be able to extend the proof of proposition 4 to the case $N \Rightarrow N+1$.
Additionally let us relax the assumption that $\alpha=0$.
<span class="theorem">
Suppose that a controlled system is $(N, \alpha, \omega)$-MO. Then, it is also $(N+1, \alpha, \omega)$-MO.
</span>
<span class="proof">
Let $\theta \neq \theta'$ be two **different** paths of length $N'+ 1$ with
$N' \dfn N-\alpha-\omega$
and let $\underline{\lambda}, \underline{\lambda}' \in \W^{\alpha}$
be any two paths of length $\alpha$ and $\bar{\lambda}, \bar{\lambda'} \in \W^{\omega}$
be any two paths of length $\omega$.
Let $\pi \dfn \underline{\lambda} \theta \bar{\lambda}, \pi'\dfn \underline{\lambda'} \theta' \bar{\lambda'} \in \W^{N+1}$ denote the concatenations of these paths.
It suffices to show that these paths are mutually discernable.
Note that since $\theta \neq \theta'$, at least one the following cases applies.<br>
**case 1.**
($\seq{\theta}{0}{N'-1} \neq \seq{\theta}{0}{N'-1}'$): $\pi$ and $\pi'$
can be written as
$\underline{\lambda} \seq{\theta}{0}{N'-1} \mu$
and
$\underline{\lambda'} \seq{\theta'}{0}{N'-1}\mu'$,
respectively,
with 
$\mu \dfn \theta_{N'} \bar{\lambda}$
and
$\mu' \dfn \theta'_{N'} \bar{\lambda'}$.
By assumption, we have that
$$
    c(\underline{\lambda}\seq{\theta}{0}{N'-1} \seq{\mu}{0}{\omega-1}, \underline{\lambda'}\seq{\theta}{0}{N'-1}' \seq{\mu'}{0}{\omega-1}, u) = \emptyset,
$$
for almost every $u \in \Re^{\na (N-1)}$. Furthermore, 
since we've shown above that, 
$$
    c(\pi, \pi', u) =  c(\underline{\lambda}\seq{\theta}{0}{N'-1} \mu, \underline{\lambda'}\seq{\theta}{0}{N'-1}' \mu', u) \subseteq c(\underline{\lambda}\seq{\theta}{0}{N'-1} \seq{\mu}{0}{\omega-1}, \underline{\lambda}\seq{\theta}{0}{N'-1}' \seq{\mu'}{0}{\omega-1}, \seq{u}{0}{N-1})
$$
for all $u \in \Re^{\na N}$, we conclude that $c(\pi, \pi', u) = \emptyset$ for almost every $u$. Hence, the paths $\pi$ and $\pi'$ are discernible.<br>
**case 2.**
($\seq{\theta}{1}{N'} \neq \seq{\theta}{1}{N'}'$):
We can write
$\seq{\pi}{1}{N} = \mu \seq{\theta}{1}{N'} \bar{\lambda}$,
$\seq{\pi'}{1}{N} = \mu' \seq{\theta'}{1}{N'} \bar{\lambda'}$,
with
$\mu \dfn \seq{\underline{\lambda}}{1}{\alpha} \theta_{0}$
and
$\mu' \dfn \seq{\underline{\lambda'}}{1}{\alpha} \theta_{0}'$.
Since $\mu, \mu' \in \W^{\alpha}$,
$\seq{\theta}{1}{N'} \neq \seq{\theta}{1}{N'} \in \W^{N'}$
and 
$\bar{\lambda}, \bar{\lambda'} \in \W^{\omega}$, 
we have that 
$$
    c(\seq{\pi}{1}{N}, \seq{\pi'}{1}{N}, u) = \emptyset
$$
for almost every $u \in \Re^{\na (N-1)}$.
Moreover, by the technical result shown above in [[#^lemma2]], we have 
$$
    \begin{aligned}
      c(\pi, \pi', u)
      {}&\subseteq{}
        T_{\pi_0,u_0}^{-1}(c(\seq{\pi}{1}{N}, \seq{\pi'}{1}{N}, \seq{u}{1}{N-1}), 
    \end{aligned}
$$
thus $c(\pi, \pi', u) = \emptyset$ for almost every $u \in \Re^{\na N}$, 
as required. This concludes the proof.
</span>
<span class="sidenote">Note on the 'almost every'-specifier. 
$c(\seq{\pi}{1}{N}, \seq{\pi'}{1}{N}, u) = \emptyset$ for almost every 
$u \in \Re^{\na N}$ can be written more explicitly by defining
$K_{N} \dfn \{ u \in \Re^{\na N} \mid c(\seq{\pi}{1}{N}, \seq{\pi'}{1}{N}, u) \neq \emptyset \}$
and stating that its Lebesgue measure $m^{\na N}(K_N) = 0$. Then, defining analogously 
$K_{N+1} \dfn \{ u \in \Re^{\na (N+1)} \mid c(\pi, \pi', u) \neq \emptyset \}$, we 
have shown above that $K_{N+1} \subset K_{N} \times \Re^{\na}$, and therefore 
$m^{(N+1)\na}(K_{N+1}) \leq m^{N \na}(K_{N}) m^{\na}(\Re^{\na})$. 
</span>

---
## old notes and scraps 

Let us write 
$\tilde \theta, \tilde \theta' \in \W^{N+2}$ as $\lambda \theta$ and $\lambda' \theta'$ with $\lambda, \lambda' \in \W$ and $\theta, \theta' \in \W^{N+1}$. 

$$
\obs(\lambda \theta) = \regmatrix{ C_{\lambda} \\ \obs(\theta) A_{\lambda}  }
$$
Alternatively, we may write $\tilde \theta = \theta \lambda$ 
$$
\obs(\theta \lambda) = \regmatrix{ \obs(\theta) \\ C_{\lambda} \Phi(\theta)}
$$
with $\Phi(\theta) = A_{\theta_N}\times \dots \times A_{\theta_0}.$
From $(N, \alpha, \omega)$-MO, we know that 
$$
\rank(\regmatrix{ \obs(\theta) & \obs(\theta') \\ 
C_{\lambda} \Phi(\theta) &  C_{\lambda'}\Phi(\theta')}) = 2n 
$$
for all $\theta, \theta'$ such that $\theta_{\alpha:N-\omega} \neq \theta_{\alpha:N-\omega}'$.
<span class="theorem">
Suppose that a controlled system is $(1+N', 0, N')$-MO, that is, 
for all paths $\theta, \theta' \in \W^{N'+1}$ with $\theta_0 \neq \theta_0'$,
$$
    \obs(\theta) x + G(\theta) u
    {}\nfd{}
    \obsaug(\theta, x, u)
     {}\neq{}
    \obsaug(\theta', x', u),
    \quad \forall x,x' \in \Re^{\ns}, 
$$
for almost every $u \in \Re^{\na N'}$. Then, $(N+N', 0, N')$-MO for any $N > 1$.
</span>
<span class="proof">
Let $\theta \neq \theta'$ be two **different** paths of length $N$. 
Let $\lambda$ and $\lambda'$ be any two paths of length $N'$. 
Then, there are two cases.<br>
**case 1.** ($\theta_0 \neq \theta_0'$): $\theta \lambda$ and $\theta' \lambda'$ can be written as $\theta_0 \mu$ and $\theta_{0}'\mu'$, respectively,
with $\mu \dfn \seq{\theta}{1}{N} \lambda$ and $\mu' \dfn \seq{\theta'}{1}{N}\lambda'$. By assumption, we have that
$$
    c(\theta_0 \seq{\mu}{0}{N'-1}, \theta'_0 \seq{\mu'}{0}{N'-1}, u) = \emptyset,
$$
for almost every $u \in \Re^{\na N'}$. Furthermore, 
since 
$$
    c(\theta \lambda, \theta' \lambda', u) = c(\theta_0 \mu, \theta'_0 \mu', u) \subseteq c(\theta_0 \seq{\mu}{0}{N'-1}, \theta'_0 \seq{\mu'}{0}{N'-1}, \seq{u}{0}{N'-1})
$$
for all $u \in \Re^{\na (N+N'-1)}$, we conclude that $c(\theta\lambda, \theta'\lambda', u) = \emptyset$ for almost every $u$, hence, the paths $\theta \lambda$ and $\theta' \lambda'$ are discernible.<br>
**case 2.** ($\theta_0 = \theta_0'$): We can write
^inclusion
$$
    \begin{aligned}
      c(\theta\lambda, \theta'\lambda')
      {}&\subseteq{}
          \hat{c}(\theta_{0}, \theta_{0}') 
          \cap 
          c(\seq{\theta}{1}{N}\lambda, \seq{\theta'}{1}{N}\lambda', \seq{u}{1}{N+N'-1})\\
     {}&\subseteq{}\cap_{i=0}^{j} \hat{c}(\theta_i, \theta_i') \cap c(\seq{\theta}{j+1}{N} \lambda, \seq{\theta'}{j+1}{N} \lambda')
    \end{aligned}
$$
for all $j < N$, where the second inclusion follows from repeated application of [[#^inclusion]]. Since $\theta \neq \theta'$, there exists a $j \leq N$ such that 
$\theta_j \neq \theta_{j}'$ so that $c(\seq{\theta}{j+1:N})$. Then, by the same argument as in case 1, we can conclude that $c(\seq{\theta}{j+1}{N} \lambda, \seq{\theta'}{j+1}{N} \lambda') = \emptyset$ for allmost every $u$. Therefore, necessarily $c(\theta \lambda, \theta' \lambda') = \emptyset$ for almost every $u$.
<br>
Therefore, all pairs of paths $\theta \lambda$ and $\theta' \lambda'$ of length $N+N'$ with $\theta \neq \theta' \in N$ are discernible and hence, the system is $(N+N', 0, N')$-MO.
</span>




