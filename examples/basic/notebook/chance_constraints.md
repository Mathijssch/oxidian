#Subjects/Optimization/Modeling #Subjects/Optimization/StochasticProgramming 

# Sigmoid approximation for chance constraints 

Our aim is to approximate the chance constraint 
$$
\prob[z > 0] = \sum_{i=1}^d p_{i} \1_{\Re_{+}}(z_{i}) \leq \delta.
$$
We consider approximations of the form 
$$
    \sigma(z; a, \beta, \overline{z}) = \frac{a}{1 + \exp(-\beta (z - \overline{z}))}.
$$
In order to ensure that $\sigma(z; a, \beta, \overline{z}) \geq \1_{\Re_{+}}(z)$, we set $\1_{\Re_{+}}(0) = \sigma(0; a, \beta, \overline{z})$,
$$
    1 = \frac{a}{1 + \exp(\beta\overline{z}))} \iff {1 + \exp(\beta\overline{z})} = a.
$$
Indeed, since $\sigma(z; a, \beta, \overline{z}) >0$, we have $\sigma(z; a, \beta, \overline{z})>\1_{\Re_{+}}(z)$ for all $z \leq 0$, and since $\sigma(z; a, \beta, \overline{z})$ is strictly monotone increasing, we have $\sigma(z; a, \beta, \overline{z}) > 1 = \1_{\Re_{+}}$ $\forall z>0$.


We can now optimize over $\overline{z}$ (and $\beta$) to obtain the tightest possible overapproximation. Our goal is to impose
$$
\begin{aligned}
    \min_{\overline{z} \in \Re, \beta>0}&
    \E_{p}[\sigma(z; a, \beta, \overline{z})] =
    \min_{\overline{z} \in \Re, \beta>0}\sum_{i=1}^{\nModes} p_{i} \frac{1 + \exp(\beta \overline{z})}{1 + \exp(-\beta z_{i}) \exp(\beta \overline{z})} \leq \gamma.
\end{aligned}
$$
Introducing the change of variables $y = \exp(\beta \overline{z}) \iff \overline{z} = \tfrac{1}{\beta} \log(y), y>0$, this is 
$$
\begin{aligned}
    &\min_{\beta > 0, y>0}&&\sum_{i=1}^{\nModes} \frac{p_{i} (1 + y)}{1 + \exp(-\beta z_{i}) y} \leq \gamma.
\end{aligned}
$$
Even for fixed $\beta$, this problem is nonconvex in $y$. However, it can be incorporated into the control problem by taking $z$ and $\beta$ to be free variables to obtain a safe approximation. That is, 
$$
\begin{aligned}
    \exists \beta > 0, y>0: \sum_{i=1}^\nModes \frac{p_{i} (1 + y)}{1 + \exp(-\beta z_{i}) y} &\leq \gamma \implies \prob_{p}[z > 0] \leq \gamma
\end{aligned}
$$
Thus, we have obtained a safe, smooth (however nonconvex) surrogate for the chance constraint.

